//! `lnix lint` — validate packages via `nix eval`.

use lnix_domain::{PackageName, ValidationResult, classify_nix_eval_error};

use crate::deps::Deps;
use crate::error::ApplicationError;
use crate::event::UseCaseEvent;

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
        d.reporter.report(&UseCaseEvent::NoPackagesToValidate);
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

    let exit_code = if result.errors.is_empty() { 0 } else { 1 };
    d.reporter
        .report(&UseCaseEvent::ValidationReport { result, verbose });

    Ok(exit_code)
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
