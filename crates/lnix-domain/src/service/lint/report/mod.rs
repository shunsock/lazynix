//! User-friendly formatting of validation results.
//!
//! The public entry points decide success vs. failure and verbosity;
//! the per-error-category message bodies live in [`messages`].

mod messages;

use super::result::ValidationResult;
use messages::{format_error_sections, format_success_message};

/// Formats a validation result for the user.
///
/// Returns a success summary when there are no errors, otherwise the
/// grouped error sections.
pub fn format_validation_result(result: &ValidationResult) -> String {
    if result.errors.is_empty() {
        return format_success_message(result.valid_packages.len());
    }
    format_error_sections(&result.errors)
}

/// Like [`format_validation_result`] but appends the raw `Debug` form of
/// each error for troubleshooting.
pub fn format_validation_result_verbose(result: &ValidationResult) -> String {
    let mut output = format_validation_result(result);

    if !result.errors.is_empty() {
        output.push_str("\n--- Verbose Error Details ---\n");
        for error in &result.errors {
            output.push_str(&format!("{:?}\n", error));
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::error::PackageValidationError;

    #[test]
    fn reports_success_with_package_count() {
        // Arrange
        let result = ValidationResult {
            valid_packages: vec!["vim".to_string(), "git".to_string()],
            errors: vec![],
        };

        // Act
        let output = format_validation_result(&result);

        // Assert
        assert!(output.contains("✓"));
        assert!(output.contains("2 package(s)"));
    }

    #[test]
    fn verbose_appends_raw_error_details() {
        // Arrange
        let result = ValidationResult {
            valid_packages: vec![],
            errors: vec![PackageValidationError::PackageNotFound {
                package: "test".to_string(),
            }],
        };

        // Act
        let output = format_validation_result_verbose(&result);

        // Assert
        assert!(output.contains("Verbose Error Details"));
        assert!(output.contains("PackageNotFound"));
    }
}
