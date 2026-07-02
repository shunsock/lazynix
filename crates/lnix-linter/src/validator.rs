//! Parallel package validation orchestrator

use crate::error::ValidationError;
use crate::nix_eval::{eval_package, eval_package_for_arch};
use lnix_domain::{PackageName, ValidationResult, classify_nix_eval_error};
use rayon::prelude::*;

/// Validates multiple packages in parallel
///
/// # Arguments
/// * `packages` - Packages to validate
/// * `target_arch` - Optional target architecture (e.g., "aarch64-darwin", "x86_64-linux")
///   If None, validates against current system architecture
///
/// # Returns
/// `ValidationResult` containing valid packages and errors
///
/// # Example
/// ```no_run
/// use lnix_domain::PackageName;
/// use lnix_linter::validator::validate_packages;
///
/// let packages: Vec<PackageName> =
///     vec!["vim".parse().unwrap(), "git".parse().unwrap()];
/// let result = validate_packages(&packages, None);
/// println!("Valid: {}, Errors: {}", result.valid_packages.len(), result.errors.len());
/// ```
pub fn validate_packages(packages: &[PackageName], target_arch: Option<&str>) -> ValidationResult {
    let results: Vec<_> = packages
        .par_iter()
        .map(|package| validate_single_package(package, target_arch))
        .collect();

    let mut valid_packages = Vec::new();
    let mut errors = Vec::new();

    for (package, result) in results {
        match result {
            Ok(()) => valid_packages.push(package),
            Err(err) => errors.push(err),
        }
    }

    ValidationResult {
        valid_packages,
        errors,
    }
}

/// Validates a single package
///
/// Returns Ok(()) if package is valid, Err(ValidationError) otherwise
fn validate_single_package(
    package: &PackageName,
    target_arch: Option<&str>,
) -> (String, Result<(), ValidationError>) {
    let eval_result = if let Some(arch) = target_arch {
        eval_package_for_arch(package, arch)
    } else {
        eval_package(package)
    };

    let validation_result = match eval_result {
        Ok(result) => {
            if result.success {
                Ok(())
            } else {
                Err(classify_nix_eval_error(package.as_str(), &result.stderr))
            }
        }
        Err(linter_error) => {
            Err(ValidationError::UnknownError {
                package: package.to_string(),
                message: linter_error.to_string(),
            })
        }
    };

    (package.to_string(), validation_result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn packages(names: &[&str]) -> Vec<PackageName> {
        names.iter().map(|name| name.parse().unwrap()).collect()
    }

    #[test]
    fn validates_existing_packages() {
        // Arrange
        let targets = packages(&["hello", "vim"]);

        // Act
        let result = validate_packages(&targets, None);

        // Assert
        assert_eq!(result.valid_packages.len(), 2);
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn separates_valid_and_invalid_packages() {
        // Arrange
        let targets = packages(&["hello", "nonexistent-xyz-12345"]);

        // Act
        let result = validate_packages(&targets, None);

        // Assert
        assert_eq!(result.valid_packages, vec!["hello".to_string()]);
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn reports_all_invalid_packages() {
        // Arrange
        let targets = packages(&["nonexistent-abc-111", "nonexistent-xyz-222"]);

        // Act
        let result = validate_packages(&targets, None);

        // Assert
        assert_eq!(result.valid_packages.len(), 0);
        assert_eq!(result.errors.len(), 2);
    }

    #[test]
    fn handles_empty_package_list() {
        // Arrange
        let targets: Vec<PackageName> = vec![];

        // Act
        let result = validate_packages(&targets, None);

        // Assert
        assert_eq!(result.valid_packages.len(), 0);
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn validates_against_explicit_architecture() {
        // Arrange
        let targets = packages(&["hello"]);

        // Act
        let result = validate_packages(&targets, Some("x86_64-linux"));

        // Assert: result may vary by system, but exactly one outcome is recorded
        assert_eq!(result.valid_packages.len() + result.errors.len(), 1);
    }
}
