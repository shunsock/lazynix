//! Parallel package validation orchestrator

use crate::error::ValidationError;
use crate::error_classifier::classify_nix_eval_error;
use crate::nix_eval::{eval_package, eval_package_for_arch};
use rayon::prelude::*;

/// Result of validating multiple packages
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Packages that passed validation
    pub valid_packages: Vec<String>,
    /// Validation errors encountered
    pub errors: Vec<ValidationError>,
}

/// Validates multiple packages in parallel
///
/// # Arguments
/// * `packages` - List of package names to validate
/// * `target_arch` - Optional target architecture (e.g., "aarch64-darwin", "x86_64-linux")
///   If None, validates against current system architecture
///
/// # Returns
/// `ValidationResult` containing valid packages and errors
///
/// # Example
/// ```no_run
/// use lnix_linter::validator::validate_packages;
///
/// let packages = vec!["vim".to_string(), "git".to_string()];
/// let result = validate_packages(&packages, None);
/// println!("Valid: {}, Errors: {}", result.valid_packages.len(), result.errors.len());
/// ```
pub fn validate_packages(packages: &[String], target_arch: Option<&str>) -> ValidationResult {
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
    package: &str,
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
                // Classify the error based on stderr
                Err(classify_nix_eval_error(package, &result.stderr))
            }
        }
        Err(linter_error) => {
            // Convert LinterError to ValidationError
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

    #[test]
    fn test_validate_all_valid_packages() {
        let packages = vec!["hello".to_string(), "vim".to_string()];
        let result = validate_packages(&packages, None);
        assert_eq!(result.valid_packages.len(), 2);
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn test_validate_with_invalid_packages() {
        let packages = vec!["hello".to_string(), "nonexistent-xyz-12345".to_string()];
        let result = validate_packages(&packages, None);
        assert_eq!(result.valid_packages.len(), 1);
        assert_eq!(result.errors.len(), 1);
        assert!(result.valid_packages.contains(&"hello".to_string()));
    }

    #[test]
    fn test_validate_all_invalid_packages() {
        let packages = vec![
            "nonexistent-abc-111".to_string(),
            "nonexistent-xyz-222".to_string(),
        ];
        let result = validate_packages(&packages, None);
        assert_eq!(result.valid_packages.len(), 0);
        assert_eq!(result.errors.len(), 2);
    }

    #[test]
    fn test_validate_empty_package_list() {
        let packages: Vec<String> = vec![];
        let result = validate_packages(&packages, None);
        assert_eq!(result.valid_packages.len(), 0);
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn test_validate_with_architecture() {
        let packages = vec!["hello".to_string()];
        let result = validate_packages(&packages, Some("x86_64-linux"));
        // hello should be available on x86_64-linux
        // Result may vary by system, but should complete without errors
        assert!(result.valid_packages.len() + result.errors.len() == 1);
    }
}
