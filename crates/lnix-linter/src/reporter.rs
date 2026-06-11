//! User-friendly error message formatting

use crate::error::ValidationError;
use crate::validator::ValidationResult;
use std::collections::HashMap;

/// Formats validation results into user-friendly error messages
///
/// # Arguments
/// * `result` - Validation result containing valid packages and errors
///
/// # Returns
/// Formatted string ready to print to stdout/stderr
pub fn format_validation_result(result: &ValidationResult) -> String {
    if result.errors.is_empty() {
        return format_success_message(result.valid_packages.len());
    }

    let mut output = String::new();

    // Group errors by type
    let (not_found, arch_unsupported, unknown) = group_errors(&result.errors);

    // Format PACKAGE_NOT_FOUND errors
    if !not_found.is_empty() {
        output.push_str(&format_package_not_found_error(&not_found));
        output.push('\n');
    }

    // Format ARCHITECTURE_UNSUPPORTED errors
    if !arch_unsupported.is_empty() {
        output.push_str(&format_architecture_unsupported_error(&arch_unsupported));
        output.push('\n');
    }

    // Format UNKNOWN errors
    if !unknown.is_empty() {
        output.push_str(&format_unknown_errors(&unknown));
    }

    output
}

/// Formats validation result with verbose mode (includes raw error details)
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

/// Formats success message
fn format_success_message(count: usize) -> String {
    format!("✓ All {} package(s) validated successfully!\n", count)
}

type ErrorGroups<'a> = (
    Vec<&'a str>,
    Vec<(&'a str, &'a str)>,
    Vec<(&'a str, &'a str)>,
);

/// Groups errors by type
fn group_errors(errors: &[ValidationError]) -> ErrorGroups<'_> {
    let mut not_found = Vec::new();
    let mut arch_unsupported = Vec::new();
    let mut unknown = Vec::new();

    for error in errors {
        match error {
            ValidationError::PackageNotFound { package } => {
                not_found.push(package.as_str());
            }
            ValidationError::ArchitectureUnsupported { package, arch } => {
                arch_unsupported.push((package.as_str(), arch.as_str()));
            }
            ValidationError::UnknownError { package, message } => {
                unknown.push((package.as_str(), message.as_str()));
            }
        }
    }

    (not_found, arch_unsupported, unknown)
}

/// Formats PACKAGE_NOT_FOUND error message
fn format_package_not_found_error(packages: &[&str]) -> String {
    let mut output = String::new();
    output.push_str("LazyNix Linting Error: PACKAGE_NOT_FOUND\n");
    output.push_str("LazyNix could not find package from registry\n\n");

    for pkg in packages {
        output.push_str(&format!("- {}\n", pkg));
    }

    output.push_str("\nSee: https://search.nixos.org/packages\n");
    output
}

/// Formats ARCHITECTURE_UNSUPPORTED error message
fn format_architecture_unsupported_error(packages: &[(&str, &str)]) -> String {
    // Group by architecture
    let mut by_arch: HashMap<&str, Vec<&str>> = HashMap::new();
    for (pkg, arch) in packages {
        by_arch.entry(arch).or_default().push(pkg);
    }

    let mut output = String::new();

    for (arch, pkgs) in by_arch {
        output
            .push_str("LazyNix Linting Error: PACKAGE_DOES_NOT_PROVIDE_TO_SELECTED_ARCHITECTURE\n");
        output.push_str(&format!(
            "LazyNix could not find package FOR YOUR ARCHITECTURE ({})\n\n",
            arch
        ));

        for pkg in pkgs {
            output.push_str(&format!("- {}\n", pkg));
        }

        output.push_str("\nSee: https://search.nixos.org/packages\n");
    }

    output
}

/// Formats UNKNOWN error messages
fn format_unknown_errors(errors: &[(&str, &str)]) -> String {
    let mut output = String::new();
    output.push_str("LazyNix Linting Error: UNKNOWN_ERROR\n");
    output.push_str("LazyNix encountered unexpected errors\n\n");

    for (pkg, message) in errors {
        output.push_str(&format!("- {}: {}\n", pkg, message));
    }

    output.push('\n');
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_success_message() {
        let result = ValidationResult {
            valid_packages: vec!["vim".to_string(), "git".to_string()],
            errors: vec![],
        };
        let output = format_validation_result(&result);
        assert!(output.contains("✓"));
        assert!(output.contains("2 package(s)"));
        assert!(output.contains("successfully"));
    }

    #[test]
    fn test_format_package_not_found() {
        let result = ValidationResult {
            valid_packages: vec![],
            errors: vec![
                ValidationError::PackageNotFound {
                    package: "nonexistent1".to_string(),
                },
                ValidationError::PackageNotFound {
                    package: "nonexistent2".to_string(),
                },
            ],
        };
        let output = format_validation_result(&result);
        assert!(output.contains("PACKAGE_NOT_FOUND"));
        assert!(output.contains("nonexistent1"));
        assert!(output.contains("nonexistent2"));
        assert!(output.contains("https://search.nixos.org/packages"));
    }

    #[test]
    fn test_format_architecture_unsupported() {
        let result = ValidationResult {
            valid_packages: vec![],
            errors: vec![ValidationError::ArchitectureUnsupported {
                package: "chromium".to_string(),
                arch: "aarch64-darwin".to_string(),
            }],
        };
        let output = format_validation_result(&result);
        assert!(output.contains("ARCHITECTURE"));
        assert!(output.contains("chromium"));
        assert!(output.contains("aarch64-darwin"));
    }

    #[test]
    fn test_format_unknown_error() {
        let result = ValidationResult {
            valid_packages: vec![],
            errors: vec![ValidationError::UnknownError {
                package: "somepkg".to_string(),
                message: "strange error occurred".to_string(),
            }],
        };
        let output = format_validation_result(&result);
        assert!(output.contains("UNKNOWN_ERROR"));
        assert!(output.contains("somepkg"));
        assert!(output.contains("strange error"));
    }

    #[test]
    fn test_format_mixed_errors() {
        let result = ValidationResult {
            valid_packages: vec!["vim".to_string()],
            errors: vec![
                ValidationError::PackageNotFound {
                    package: "pkg1".to_string(),
                },
                ValidationError::ArchitectureUnsupported {
                    package: "pkg2".to_string(),
                    arch: "x86_64-linux".to_string(),
                },
            ],
        };
        let output = format_validation_result(&result);
        assert!(output.contains("PACKAGE_NOT_FOUND"));
        assert!(output.contains("ARCHITECTURE"));
        assert!(output.contains("pkg1"));
        assert!(output.contains("pkg2"));
    }

    #[test]
    fn test_format_verbose() {
        let result = ValidationResult {
            valid_packages: vec![],
            errors: vec![ValidationError::PackageNotFound {
                package: "test".to_string(),
            }],
        };
        let output = format_validation_result_verbose(&result);
        assert!(output.contains("Verbose Error Details"));
        assert!(output.contains("PackageNotFound"));
    }
}
