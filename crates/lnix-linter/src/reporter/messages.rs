//! Per-category error message bodies and the success summary.

use crate::error::ValidationError;
use std::collections::HashMap;

/// Success summary line.
pub(super) fn format_success_message(count: usize) -> String {
    format!("✓ All {} package(s) validated successfully!\n", count)
}

/// Renders every non-empty error category into one combined report.
pub(super) fn format_error_sections(errors: &[ValidationError]) -> String {
    let (not_found, arch_unsupported, unknown) = group_errors(errors);

    let mut output = String::new();
    if !not_found.is_empty() {
        output.push_str(&format_package_not_found(&not_found));
        output.push('\n');
    }
    if !arch_unsupported.is_empty() {
        output.push_str(&format_architecture_unsupported(&arch_unsupported));
        output.push('\n');
    }
    if !unknown.is_empty() {
        output.push_str(&format_unknown(&unknown));
    }
    output
}

type ErrorGroups<'a> = (
    Vec<&'a str>,
    Vec<(&'a str, &'a str)>,
    Vec<(&'a str, &'a str)>,
);

/// Partitions errors by category, borrowing their fields.
fn group_errors(errors: &[ValidationError]) -> ErrorGroups<'_> {
    let mut not_found = Vec::new();
    let mut arch_unsupported = Vec::new();
    let mut unknown = Vec::new();

    for error in errors {
        match error {
            ValidationError::PackageNotFound { package } => not_found.push(package.as_str()),
            ValidationError::ArchitectureUnsupported { package, arch } => {
                arch_unsupported.push((package.as_str(), arch.as_str()))
            }
            ValidationError::UnknownError { package, message } => {
                unknown.push((package.as_str(), message.as_str()))
            }
        }
    }

    (not_found, arch_unsupported, unknown)
}

fn format_package_not_found(packages: &[&str]) -> String {
    let mut output = String::new();
    output.push_str("LazyNix Linting Error: PACKAGE_NOT_FOUND\n");
    output.push_str("LazyNix could not find package from registry\n\n");
    for pkg in packages {
        output.push_str(&format!("- {}\n", pkg));
    }
    output.push_str("\nSee: https://search.nixos.org/packages\n");
    output
}

fn format_architecture_unsupported(packages: &[(&str, &str)]) -> String {
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

fn format_unknown(errors: &[(&str, &str)]) -> String {
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
    fn package_not_found_lists_each_package_and_link() {
        // Arrange
        let errors = vec![
            ValidationError::PackageNotFound {
                package: "nonexistent1".to_string(),
            },
            ValidationError::PackageNotFound {
                package: "nonexistent2".to_string(),
            },
        ];

        // Act
        let output = format_error_sections(&errors);

        // Assert
        assert!(output.contains("PACKAGE_NOT_FOUND"));
        assert!(output.contains("nonexistent1"));
        assert!(output.contains("nonexistent2"));
        assert!(output.contains("https://search.nixos.org/packages"));
    }

    #[test]
    fn architecture_unsupported_names_package_and_arch() {
        // Arrange
        let errors = vec![ValidationError::ArchitectureUnsupported {
            package: "chromium".to_string(),
            arch: "aarch64-darwin".to_string(),
        }];

        // Act
        let output = format_error_sections(&errors);

        // Assert
        assert!(output.contains("ARCHITECTURE"));
        assert!(output.contains("chromium"));
        assert!(output.contains("aarch64-darwin"));
    }

    #[test]
    fn combines_distinct_error_categories() {
        // Arrange
        let errors = vec![
            ValidationError::PackageNotFound {
                package: "pkg1".to_string(),
            },
            ValidationError::ArchitectureUnsupported {
                package: "pkg2".to_string(),
                arch: "x86_64-linux".to_string(),
            },
        ];

        // Act
        let output = format_error_sections(&errors);

        // Assert
        assert!(output.contains("PACKAGE_NOT_FOUND"));
        assert!(output.contains("ARCHITECTURE"));
        assert!(output.contains("pkg1"));
        assert!(output.contains("pkg2"));
    }

    #[test]
    fn unknown_error_includes_package_and_message() {
        // Arrange
        let errors = vec![ValidationError::UnknownError {
            package: "somepkg".to_string(),
            message: "strange error occurred".to_string(),
        }];

        // Act
        let output = format_error_sections(&errors);

        // Assert
        assert!(output.contains("UNKNOWN_ERROR"));
        assert!(output.contains("somepkg"));
        assert!(output.contains("strange error"));
    }
}
