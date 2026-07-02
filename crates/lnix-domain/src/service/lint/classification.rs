//! Error classification for nix eval output

use super::error::PackageValidationError;

/// Classifies nix eval error output into structured error types
///
/// # Arguments
/// * `package` - Name of the package being evaluated
/// * `stderr` - Standard error output from `nix eval` command
///
/// # Returns
/// A `PackageValidationError` variant based on the error message pattern
///
/// # Examples
/// ```
/// use lnix_domain::{PackageValidationError, classify_nix_eval_error};
///
/// let stderr = "error: attribute 'foo' does not provide attribute 'outPath'";
/// let result = classify_nix_eval_error("foo", stderr);
/// assert!(matches!(result, PackageValidationError::PackageNotFound { .. }));
/// ```
pub fn classify_nix_eval_error(package: &str, stderr: &str) -> PackageValidationError {
    if stderr.contains("does not provide attribute") || stderr.contains("does not provide") {
        return PackageValidationError::PackageNotFound {
            package: package.to_string(),
        };
    }

    if stderr.contains("is not available on the requested hostPlatform")
        || stderr.contains("not available on the requested hostPlatform")
    {
        let arch = extract_architecture(stderr).unwrap_or_else(|| "unknown".to_string());
        return PackageValidationError::ArchitectureUnsupported {
            package: package.to_string(),
            arch,
        };
    }

    if stderr.contains("unsupported system") || stderr.contains("is not supported") {
        let arch = extract_architecture(stderr).unwrap_or_else(|| "unknown".to_string());
        return PackageValidationError::ArchitectureUnsupported {
            package: package.to_string(),
            arch,
        };
    }

    PackageValidationError::UnknownError {
        package: package.to_string(),
        message: extract_error_message(stderr),
    }
}

/// Extracts architecture information from error message
///
/// Looks for common architecture patterns like "aarch64-darwin", "x86_64-linux", etc.
fn extract_architecture(stderr: &str) -> Option<String> {
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
            PackageValidationError::PackageNotFound {
                package: "nonexistent".to_string()
            }
        );
    }

    #[test]
    fn test_classify_package_not_found_variant() {
        let stderr = "attribute 'xyz' does not provide";
        let result = classify_nix_eval_error("xyz", stderr);
        assert!(matches!(
            result,
            PackageValidationError::PackageNotFound { .. }
        ));
    }

    #[test]
    fn test_classify_architecture_unsupported() {
        let stderr =
            "error: chromium is not available on the requested hostPlatform: aarch64-darwin";
        let result = classify_nix_eval_error("chromium", stderr);
        assert_eq!(
            result,
            PackageValidationError::ArchitectureUnsupported {
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
            PackageValidationError::ArchitectureUnsupported { .. }
        ));
    }

    #[test]
    fn test_classify_unsupported_system() {
        let stderr = "unsupported system: x86_64-linux";
        let result = classify_nix_eval_error("pkg", stderr);
        assert_eq!(
            result,
            PackageValidationError::ArchitectureUnsupported {
                package: "pkg".to_string(),
                arch: "x86_64-linux".to_string()
            }
        );
    }

    #[test]
    fn test_classify_unknown_error() {
        let stderr = "some random error message";
        let result = classify_nix_eval_error("package", stderr);
        assert!(matches!(
            result,
            PackageValidationError::UnknownError { .. }
        ));
        if let PackageValidationError::UnknownError { package, message } = result {
            assert_eq!(package, "package");
            assert!(message.contains("some random error message"));
        }
    }

    #[test]
    fn test_classify_empty_stderr() {
        let stderr = "";
        let result = classify_nix_eval_error("package", stderr);
        assert!(matches!(
            result,
            PackageValidationError::UnknownError { .. }
        ));
    }

    #[test]
    fn test_classify_multiline_error() {
        let stderr = "error: some error\nerror: attribute 'pkg' does not provide attribute 'outPath'\nother info";
        let result = classify_nix_eval_error("pkg", stderr);
        assert!(matches!(
            result,
            PackageValidationError::PackageNotFound { .. }
        ));
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
