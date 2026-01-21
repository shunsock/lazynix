//! Error classification for nix eval output

use crate::error::ValidationError;

/// Classifies nix eval error output into structured error types
///
/// # Arguments
/// * `package` - Name of the package being evaluated
/// * `stderr` - Standard error output from `nix eval` command
///
/// # Returns
/// A `ValidationError` variant based on the error message pattern
///
/// # Examples
/// ```
/// use lnix_linter::error_classifier::classify_nix_eval_error;
/// use lnix_linter::error::ValidationError;
///
/// let stderr = "error: attribute 'foo' does not provide attribute 'outPath'";
/// let result = classify_nix_eval_error("foo", stderr);
/// assert!(matches!(result, ValidationError::PackageNotFound { .. }));
/// ```
pub fn classify_nix_eval_error(package: &str, stderr: &str) -> ValidationError {
    // Check for "does not provide attribute" pattern (package not found)
    if stderr.contains("does not provide attribute") || stderr.contains("does not provide") {
        return ValidationError::PackageNotFound {
            package: package.to_string(),
        };
    }

    // Check for architecture unsupported pattern
    if stderr.contains("is not available on the requested hostPlatform")
        || stderr.contains("not available on the requested hostPlatform")
    {
        // Try to extract architecture from error message or context
        let arch = extract_architecture(stderr).unwrap_or_else(|| "unknown".to_string());
        return ValidationError::ArchitectureUnsupported {
            package: package.to_string(),
            arch,
        };
    }

    // Check for unsupported system patterns
    if stderr.contains("unsupported system") || stderr.contains("is not supported") {
        let arch = extract_architecture(stderr).unwrap_or_else(|| "unknown".to_string());
        return ValidationError::ArchitectureUnsupported {
            package: package.to_string(),
            arch,
        };
    }

    // Default to unknown error
    ValidationError::UnknownError {
        package: package.to_string(),
        message: extract_error_message(stderr),
    }
}

/// Extracts architecture information from error message
///
/// Looks for common architecture patterns like "aarch64-darwin", "x86_64-linux", etc.
fn extract_architecture(stderr: &str) -> Option<String> {
    // Common architecture patterns
    let arch_patterns = [
        "aarch64-darwin",
        "x86_64-darwin",
        "x86_64-linux",
        "aarch64-linux",
        "i686-linux",
    ];

    for pattern in &arch_patterns {
        if stderr.contains(pattern) {
            return Some(pattern.to_string());
        }
    }

    None
}

/// Extracts a clean error message from stderr output
///
/// Removes common prefixes and trims whitespace
fn extract_error_message(stderr: &str) -> String {
    stderr
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            line.trim()
                .strip_prefix("error:")
                .unwrap_or(line.trim())
                .trim()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_package_not_found() {
        let stderr = "error: attribute 'nonexistent' does not provide attribute 'outPath'";
        let result = classify_nix_eval_error("nonexistent", stderr);
        assert_eq!(
            result,
            ValidationError::PackageNotFound {
                package: "nonexistent".to_string()
            }
        );
    }

    #[test]
    fn test_classify_package_not_found_variant() {
        let stderr = "attribute 'xyz' does not provide";
        let result = classify_nix_eval_error("xyz", stderr);
        assert!(matches!(result, ValidationError::PackageNotFound { .. }));
    }

    #[test]
    fn test_classify_architecture_unsupported() {
        let stderr =
            "error: chromium is not available on the requested hostPlatform: aarch64-darwin";
        let result = classify_nix_eval_error("chromium", stderr);
        assert_eq!(
            result,
            ValidationError::ArchitectureUnsupported {
                package: "chromium".to_string(),
                arch: "aarch64-darwin".to_string()
            }
        );
    }

    #[test]
    fn test_classify_architecture_unsupported_without_arch() {
        let stderr = "package is not available on the requested hostPlatform";
        let result = classify_nix_eval_error("somepackage", stderr);
        assert!(matches!(
            result,
            ValidationError::ArchitectureUnsupported { .. }
        ));
    }

    #[test]
    fn test_classify_unsupported_system() {
        let stderr = "unsupported system: x86_64-linux";
        let result = classify_nix_eval_error("pkg", stderr);
        assert_eq!(
            result,
            ValidationError::ArchitectureUnsupported {
                package: "pkg".to_string(),
                arch: "x86_64-linux".to_string()
            }
        );
    }

    #[test]
    fn test_classify_unknown_error() {
        let stderr = "some random error message";
        let result = classify_nix_eval_error("package", stderr);
        assert!(matches!(result, ValidationError::UnknownError { .. }));
        if let ValidationError::UnknownError { package, message } = result {
            assert_eq!(package, "package");
            assert!(message.contains("some random error message"));
        }
    }

    #[test]
    fn test_classify_empty_stderr() {
        let stderr = "";
        let result = classify_nix_eval_error("package", stderr);
        assert!(matches!(result, ValidationError::UnknownError { .. }));
    }

    #[test]
    fn test_classify_multiline_error() {
        let stderr = "error: some error\nerror: attribute 'pkg' does not provide attribute 'outPath'\nother info";
        let result = classify_nix_eval_error("pkg", stderr);
        assert!(matches!(result, ValidationError::PackageNotFound { .. }));
    }

    #[test]
    fn test_extract_architecture_aarch64_darwin() {
        let stderr = "error on aarch64-darwin platform";
        assert_eq!(
            extract_architecture(stderr),
            Some("aarch64-darwin".to_string())
        );
    }

    #[test]
    fn test_extract_architecture_x86_64_linux() {
        let stderr = "unsupported on x86_64-linux";
        assert_eq!(
            extract_architecture(stderr),
            Some("x86_64-linux".to_string())
        );
    }

    #[test]
    fn test_extract_architecture_not_found() {
        let stderr = "some error without architecture info";
        assert_eq!(extract_architecture(stderr), None);
    }

    #[test]
    fn test_extract_error_message_single_line() {
        let stderr = "error: something went wrong";
        let message = extract_error_message(stderr);
        assert_eq!(message, "something went wrong");
    }

    #[test]
    fn test_extract_error_message_multiline() {
        let stderr = "error: first error\nerror: second error\n";
        let message = extract_error_message(stderr);
        assert_eq!(message, "first error second error");
    }

    #[test]
    fn test_extract_error_message_with_empty_lines() {
        let stderr = "error: test\n\n\nerror: another";
        let message = extract_error_message(stderr);
        assert_eq!(message, "test another");
    }
}
