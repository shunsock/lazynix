//! `lnix lint` — validate packages via `nix eval`.

use std::collections::HashSet;

use lnix_domain::{
    NixError, PackageName, PackageValidationError, PinnedPackageEntry, ValidationResult,
    classify_nix_eval_error, format_validation_result, format_validation_result_verbose,
};

use crate::deps::Deps;
use crate::error::ApplicationError;

/// Result of verifying pinned entries against the version resolver.
/// `failed_names` lets the caller exclude broken packages from the
/// valid-count list without inspecting error variants, keeping the
/// `if let` fall-through (which is structurally unreachable) out of
/// coverage reports.
struct PinnedVerification {
    failed_names: Vec<String>,
    errors: Vec<PackageValidationError>,
}

/// Evaluates every declared package (stable + unstable + pinned) via
/// `nix eval` and, for pinned entries whose commit/attr are not yet
/// cached, additionally verifies that the requested version can be
/// resolved. Read-only: never rewrites `lazynix.yaml`.
/// Exit code 1 when any package fails.
pub fn lint(d: &Deps, verbose: bool, arch: Option<&str>) -> Result<i32, ApplicationError> {
    let config = d.repo.read_config()?;

    let package = &config.dev_shell.package;
    let channel_names = package
        .stable
        .iter()
        .chain(package.unstable.iter())
        .map(|entry| entry.name.clone());
    let pinned_names = package.pinned.iter().map(|entry| entry.name.clone());
    let packages: Vec<PackageName> = channel_names.chain(pinned_names).collect();

    if packages.is_empty() {
        d.out.info("No packages to validate.");
        return Ok(0);
    }

    let outcomes = d.nix_eval.eval_packages(&packages, arch)?;

    let mut valid_packages = Vec::new();
    let mut errors = Vec::new();
    let mut name_eval_failed = HashSet::new();
    for (name, outcome) in packages.iter().zip(outcomes) {
        if outcome.success {
            valid_packages.push(name.to_string());
        } else {
            name_eval_failed.insert(name.to_string());
            errors.push(classify_nix_eval_error(name.as_str(), &outcome.stderr));
        }
    }

    let verification = verify_pinned_versions(d, &package.pinned, &name_eval_failed)?;
    valid_packages.retain(|valid| !verification.failed_names.contains(valid));
    errors.extend(verification.errors);

    let result = ValidationResult {
        valid_packages,
        errors,
    };

    let report = if verbose {
        format_validation_result_verbose(&result)
    } else {
        format_validation_result(&result)
    };
    d.out.info(report.trim_end());

    Ok(if result.errors.is_empty() { 0 } else { 1 })
}

/// Verifies pinned entries whose commit/attr are not yet cached and
/// whose attribute name eval has not already failed. Read-only: never
/// invokes the config writer, so `lazynix.yaml` is left untouched.
///
/// Returns [`PinnedVerification`] pairing each failed package name with
/// its `VersionNotFound` error, so the caller can update the valid list
/// without pattern-matching on error variants. Infra failures from the
/// resolver (anything other than `NixError::VersionResolution`)
/// short-circuit as `Err`.
fn verify_pinned_versions(
    d: &Deps,
    pinned: &[PinnedPackageEntry],
    name_eval_failed: &HashSet<String>,
) -> Result<PinnedVerification, ApplicationError> {
    let mut failed_names = Vec::new();
    let mut errors = Vec::new();
    for entry in pinned {
        if entry.resolved_commit.is_some() && entry.resolved_attr.is_some() {
            continue;
        }
        if name_eval_failed.contains(entry.name.as_str()) {
            continue;
        }
        match d.resolver.resolve(&entry.name, &entry.version) {
            Ok(_) => {}
            Err(NixError::VersionResolution { message, .. }) => {
                failed_names.push(entry.name.to_string());
                errors.push(PackageValidationError::VersionNotFound {
                    package: entry.name.to_string(),
                    version: entry.version.to_string(),
                    message,
                });
            }
            Err(other) => return Err(other.into()),
        }
    }
    Ok(PinnedVerification {
        failed_names,
        errors,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mocks::*;

    #[test]
    fn empty_package_list_succeeds_without_evaluating() {
        // Arrange
        let m = Mocks::with_config(config_from_yaml("devShell:\n  package:\n    stable: []\n"));

        // Act
        let code = lint(&m.deps(), false, None).unwrap();

        // Assert
        assert_eq!(code, 0);
        assert!(
            m.out
                .infos()
                .contains(&"No packages to validate.".to_string())
        );
    }

    #[test]
    fn all_valid_packages_report_success_with_count() {
        // Arrange
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable:\n      - name: hello\n    unstable:\n      - name: vim\n",
        ));

        // Act
        let code = lint(&m.deps(), false, None).unwrap();

        // Assert
        assert_eq!(code, 0);
        assert!(
            m.out
                .infos()
                .iter()
                .any(|line| line.contains("✓") && line.contains("2 package(s)"))
        );
    }

    #[test]
    fn failing_package_yields_exit_1_and_categorized_report() {
        // Arrange
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable:\n      - name: hello\n      - name: ghost-pkg\n",
        ))
        .with_failing_packages(&["ghost-pkg"]);

        // Act
        let code = lint(&m.deps(), false, None).unwrap();

        // Assert
        assert_eq!(code, 1);
        let report = m.out.infos().join("\n");
        assert!(report.contains("PACKAGE_NOT_FOUND"));
        assert!(report.contains("ghost-pkg"));
    }

    #[test]
    fn pinned_only_config_is_validated_not_skipped() {
        // Arrange
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable: []\n    pinned:\n      - name: go\n        version: \"1.21.13\"\n",
        ));

        // Act
        let code = lint(&m.deps(), false, None).unwrap();

        // Assert
        assert_eq!(code, 0);
        let infos = m.out.infos();
        assert!(!infos.contains(&"No packages to validate.".to_string()));
        assert!(
            infos
                .iter()
                .any(|line| line.contains("✓") && line.contains("1 package(s)"))
        );
    }

    #[test]
    fn pinned_name_that_does_not_exist_reports_categorized_error() {
        // Arrange
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable: []\n    pinned:\n      - name: ghost-pkg\n        version: \"9.9.9\"\n",
        ))
        .with_failing_packages(&["ghost-pkg"]);

        // Act
        let code = lint(&m.deps(), false, None).unwrap();

        // Assert
        assert_eq!(code, 1);
        let report = m.out.infos().join("\n");
        assert!(report.contains("PACKAGE_NOT_FOUND"));
        assert!(report.contains("ghost-pkg"));
    }

    #[test]
    fn stable_and_pinned_are_counted_together_on_success() {
        // Arrange
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable:\n      - name: hello\n    pinned:\n      - name: go\n        version: \"1.21.13\"\n",
        ));

        // Act
        let code = lint(&m.deps(), false, None).unwrap();

        // Assert
        assert_eq!(code, 0);
        let infos = m.out.infos();
        assert!(
            infos
                .iter()
                .any(|line| line.contains("✓") && line.contains("2 package(s)"))
        );
    }

    #[test]
    fn cached_pinned_entry_skips_resolver_call() {
        // Arrange
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable: []\n    pinned:\n      - name: go\n        version: \"1.21.13\"\n        resolvedCommit: \"e607cb5\"\n        resolvedAttr: \"go_1_21\"\n",
        ));

        // Act
        let code = lint(&m.deps(), false, None).unwrap();

        // Assert
        assert_eq!(code, 0);
        assert!(m.resolver.resolve_calls().is_empty());
    }

    #[test]
    fn lint_never_persists_config_even_after_resolving_versions() {
        // Arrange
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable: []\n    pinned:\n      - name: go\n        version: \"1.21.13\"\n",
        ));

        // Act
        let _ = lint(&m.deps(), false, None).unwrap();

        // Assert
        assert_eq!(m.resolver.resolve_calls(), vec!["go".to_string()]);
        assert!(m.repo.persisted_config().is_none());
    }

    #[test]
    fn successful_version_resolution_preserves_valid_count_across_stable_and_pinned() {
        // Arrange
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable:\n      - name: hello\n    pinned:\n      - name: go\n        version: \"1.21.13\"\n",
        ));

        // Act
        let code = lint(&m.deps(), false, None).unwrap();

        // Assert
        assert_eq!(code, 0);
        assert_eq!(m.resolver.resolve_calls(), vec!["go".to_string()]);
        let infos = m.out.infos();
        assert!(
            infos
                .iter()
                .any(|line| line.contains("✓") && line.contains("2 package(s)"))
        );
    }

    #[test]
    fn pinned_with_failing_name_eval_skips_resolver_call() {
        // Arrange
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable: []\n    pinned:\n      - name: ghost-pkg\n        version: \"9.9.9\"\n",
        ))
        .with_failing_packages(&["ghost-pkg"]);

        // Act
        let code = lint(&m.deps(), false, None).unwrap();

        // Assert
        assert_eq!(code, 1);
        assert!(m.resolver.resolve_calls().is_empty());
    }

    #[test]
    fn resolver_infra_failure_propagates_as_application_error() {
        // Arrange
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable: []\n    pinned:\n      - name: go\n        version: \"1.21.13\"\n",
        ))
        .with_resolver_infra_failure();

        // Act
        let result = lint(&m.deps(), false, None);

        // Assert
        assert!(matches!(
            result,
            Err(ApplicationError::Nix(NixError::NoExitCode))
        ));
    }

    #[test]
    fn pinned_version_that_cannot_be_resolved_reports_version_not_found() {
        // Arrange
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable: []\n    pinned:\n      - name: go\n        version: \"9.9.9\"\n",
        ))
        .with_failing_versions(&[("go", "no matching commit")]);

        // Act
        let code = lint(&m.deps(), false, None).unwrap();

        // Assert
        assert_eq!(code, 1);
        let report = m.out.infos().join("\n");
        assert!(report.contains("VERSION_NOT_FOUND"));
        assert!(report.contains("go"));
        assert!(report.contains("9.9.9"));
    }

    #[test]
    fn verbose_appends_raw_error_details() {
        // Arrange
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable:\n      - name: ghost-pkg\n",
        ))
        .with_failing_packages(&["ghost-pkg"]);

        // Act
        let code = lint(&m.deps(), true, None).unwrap();

        // Assert
        assert_eq!(code, 1);
        assert!(
            m.out
                .infos()
                .iter()
                .any(|line| line.contains("Verbose Error Details"))
        );
    }
}
