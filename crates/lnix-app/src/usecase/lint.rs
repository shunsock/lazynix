//! `lnix lint` — validate packages via `nix eval`.

use lnix_domain::{
    PackageName, ValidationResult, classify_nix_eval_error, format_validation_result,
    format_validation_result_verbose,
};

use crate::deps::Deps;
use crate::error::ApplicationError;

/// Evaluates every channel-based package (stable + unstable) and
/// prints the validation report. Exit code 1 when any package fails.
pub fn lint(d: &Deps, verbose: bool, arch: Option<&str>) -> Result<i32, ApplicationError> {
    let config = d.repo.read_config()?;

    let package = &config.dev_shell.package;
    let packages: Vec<PackageName> = package
        .stable
        .iter()
        .chain(package.unstable.iter())
        .map(|entry| entry.name.clone())
        .collect();

    if packages.is_empty() {
        d.out.info("No packages to validate.");
        return Ok(0);
    }

    let outcomes = d.nix_eval.eval_packages(&packages, arch)?;

    let mut valid_packages = Vec::new();
    let mut errors = Vec::new();
    for (name, outcome) in packages.iter().zip(outcomes) {
        if outcome.success {
            valid_packages.push(name.to_string());
        } else {
            errors.push(classify_nix_eval_error(name.as_str(), &outcome.stderr));
        }
    }
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
