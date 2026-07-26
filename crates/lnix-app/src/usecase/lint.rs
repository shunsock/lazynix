//! `lnix lint` — validate packages via `nix eval`.

use std::collections::HashSet;

use lnix_domain::{
    NixError, PackageName, PackageValidationError, PinnedPackageEntry, ValidationResult,
    classify_nix_eval_error,
};

use crate::deps::Deps;
use crate::error::ApplicationError;
use crate::event::UseCaseEvent;

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
        d.reporter.report(&UseCaseEvent::NoPackagesToValidate);
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

    let exit_code = if result.errors.is_empty() { 0 } else { 1 };
    d.reporter
        .report(&UseCaseEvent::ValidationReport { result, verbose });

    Ok(exit_code)
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
    use lnix_domain::PackageValidationError;

    #[test]
    fn empty_package_list_succeeds_without_evaluating() {
        let m = Mocks::with_config(config_from_yaml("devShell:\n  package:\n    stable: []\n"));

        let code = lint(&m.deps(), false, None).unwrap();

        assert_eq!(code, 0);
        assert!(
            m.reporter
                .events()
                .iter()
                .any(|e| matches!(e, UseCaseEvent::NoPackagesToValidate))
        );
    }

    #[test]
    fn all_valid_packages_report_success_with_count() {
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable:\n      - name: hello\n    unstable:\n      - name: vim\n",
        ));

        let code = lint(&m.deps(), false, None).unwrap();

        assert_eq!(code, 0);
        let report = m
            .reporter
            .events()
            .into_iter()
            .find_map(|e| match e {
                UseCaseEvent::ValidationReport { result, .. } => Some(result),
                _ => None,
            })
            .expect("expected ValidationReport event");
        assert_eq!(report.valid_packages.len(), 2);
        assert!(report.errors.is_empty());
    }

    #[test]
    fn failing_package_yields_exit_1_and_categorized_report() {
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable:\n      - name: hello\n      - name: ghost-pkg\n",
        ))
        .with_failing_packages(&["ghost-pkg"]);

        let code = lint(&m.deps(), false, None).unwrap();

        assert_eq!(code, 1);
        let report = m
            .reporter
            .events()
            .into_iter()
            .find_map(|e| match e {
                UseCaseEvent::ValidationReport { result, .. } => Some(result),
                _ => None,
            })
            .expect("expected ValidationReport event");
        assert!(report.errors.iter().any(|err| matches!(
            err,
            PackageValidationError::PackageNotFound { package } if package == "ghost-pkg"
        )));
    }

    fn extract_report(m: &Mocks) -> ValidationResult {
        m.reporter
            .events()
            .into_iter()
            .find_map(|e| match e {
                UseCaseEvent::ValidationReport { result, .. } => Some(result),
                _ => None,
            })
            .expect("expected ValidationReport event")
    }

    #[test]
    fn pinned_only_config_is_validated_not_skipped() {
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable: []\n    pinned:\n      - name: go\n        version: \"1.21.13\"\n",
        ));

        let code = lint(&m.deps(), false, None).unwrap();

        assert_eq!(code, 0);
        assert!(
            !m.reporter
                .events()
                .iter()
                .any(|e| matches!(e, UseCaseEvent::NoPackagesToValidate))
        );
        let report = extract_report(&m);
        assert_eq!(report.valid_packages.len(), 1);
        assert!(report.errors.is_empty());
    }

    #[test]
    fn pinned_name_that_does_not_exist_reports_categorized_error() {
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable: []\n    pinned:\n      - name: ghost-pkg\n        version: \"9.9.9\"\n",
        ))
        .with_failing_packages(&["ghost-pkg"]);

        let code = lint(&m.deps(), false, None).unwrap();

        assert_eq!(code, 1);
        let report = extract_report(&m);
        assert!(report.errors.iter().any(|err| matches!(
            err,
            PackageValidationError::PackageNotFound { package } if package == "ghost-pkg"
        )));
    }

    #[test]
    fn stable_and_pinned_are_counted_together_on_success() {
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable:\n      - name: hello\n    pinned:\n      - name: go\n        version: \"1.21.13\"\n",
        ));

        let code = lint(&m.deps(), false, None).unwrap();

        assert_eq!(code, 0);
        let report = extract_report(&m);
        assert_eq!(report.valid_packages.len(), 2);
        assert!(report.errors.is_empty());
    }

    #[test]
    fn cached_pinned_entry_skips_resolver_call() {
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable: []\n    pinned:\n      - name: go\n        version: \"1.21.13\"\n        resolvedCommit: \"e607cb5\"\n        resolvedAttr: \"go_1_21\"\n",
        ));

        let code = lint(&m.deps(), false, None).unwrap();

        assert_eq!(code, 0);
        assert!(m.resolver.resolve_calls().is_empty());
    }

    #[test]
    fn lint_never_persists_config_even_after_resolving_versions() {
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable: []\n    pinned:\n      - name: go\n        version: \"1.21.13\"\n",
        ));

        let _ = lint(&m.deps(), false, None).unwrap();

        assert_eq!(m.resolver.resolve_calls(), vec!["go".to_string()]);
        assert!(m.repo.persisted_config().is_none());
    }

    #[test]
    fn successful_version_resolution_preserves_valid_count_across_stable_and_pinned() {
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable:\n      - name: hello\n    pinned:\n      - name: go\n        version: \"1.21.13\"\n",
        ));

        let code = lint(&m.deps(), false, None).unwrap();

        assert_eq!(code, 0);
        assert_eq!(m.resolver.resolve_calls(), vec!["go".to_string()]);
        let report = extract_report(&m);
        assert_eq!(report.valid_packages.len(), 2);
    }

    #[test]
    fn pinned_with_failing_name_eval_skips_resolver_call() {
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable: []\n    pinned:\n      - name: ghost-pkg\n        version: \"9.9.9\"\n",
        ))
        .with_failing_packages(&["ghost-pkg"]);

        let code = lint(&m.deps(), false, None).unwrap();

        assert_eq!(code, 1);
        assert!(m.resolver.resolve_calls().is_empty());
    }

    #[test]
    fn resolver_infra_failure_propagates_as_application_error() {
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable: []\n    pinned:\n      - name: go\n        version: \"1.21.13\"\n",
        ))
        .with_resolver_infra_failure();

        let result = lint(&m.deps(), false, None);

        assert!(matches!(
            result,
            Err(ApplicationError::Nix(NixError::NoExitCode))
        ));
    }

    #[test]
    fn pinned_version_that_cannot_be_resolved_reports_version_not_found() {
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable: []\n    pinned:\n      - name: go\n        version: \"9.9.9\"\n",
        ))
        .with_failing_versions(&[("go", "no matching commit")]);

        let code = lint(&m.deps(), false, None).unwrap();

        assert_eq!(code, 1);
        let report = extract_report(&m);
        assert!(report.errors.iter().any(|err| matches!(
            err,
            PackageValidationError::VersionNotFound { package, version, .. }
            if package == "go" && version == "9.9.9"
        )));
    }

    #[test]
    fn verbose_appends_raw_error_details() {
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable:\n      - name: ghost-pkg\n",
        ))
        .with_failing_packages(&["ghost-pkg"]);

        let code = lint(&m.deps(), true, None).unwrap();

        assert_eq!(code, 1);
        assert!(
            m.reporter
                .events()
                .iter()
                .any(|e| matches!(e, UseCaseEvent::ValidationReport { verbose: true, .. }))
        );
    }
}
