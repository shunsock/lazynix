//! Integration tests for the linter functionality

use lnix_linter::{ValidationError, format_validation_result, validate_packages};

#[test]
fn test_all_valid_packages() {
    let packages = vec!["hello".to_string(), "vim".to_string(), "git".to_string()];
    let result = validate_packages(&packages, None);

    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.valid_packages.len(), 3);
    assert!(result.valid_packages.contains(&"hello".to_string()));
    assert!(result.valid_packages.contains(&"vim".to_string()));
    assert!(result.valid_packages.contains(&"git".to_string()));
}

#[test]
fn test_non_existent_packages() {
    let packages = vec![
        "nonexistent-pkg-xyz-12345".to_string(),
        "another-fake-package-99999".to_string(),
    ];
    let result = validate_packages(&packages, None);

    assert_eq!(result.valid_packages.len(), 0);
    assert_eq!(result.errors.len(), 2);

    // Check that errors are PackageNotFound
    for error in &result.errors {
        assert!(matches!(error, ValidationError::PackageNotFound { .. }));
    }
}

#[test]
fn test_mixed_valid_invalid_packages() {
    let packages = vec![
        "hello".to_string(),
        "nonexistent-xyz".to_string(),
        "vim".to_string(),
    ];
    let result = validate_packages(&packages, None);

    assert_eq!(result.valid_packages.len(), 2);
    assert_eq!(result.errors.len(), 1);
    assert!(result.valid_packages.contains(&"hello".to_string()));
    assert!(result.valid_packages.contains(&"vim".to_string()));
}

#[test]
fn test_empty_package_list() {
    let packages: Vec<String> = vec![];
    let result = validate_packages(&packages, None);

    assert_eq!(result.valid_packages.len(), 0);
    assert_eq!(result.errors.len(), 0);
}

#[test]
fn test_architecture_specific_validation() {
    // Test with a common package on x86_64-linux
    let packages = vec!["hello".to_string()];
    let result = validate_packages(&packages, Some("x86_64-linux"));

    // Should complete without panicking
    assert!(result.valid_packages.len() + result.errors.len() == 1);
}

#[test]
fn test_performance_10_packages() {
    use std::time::Instant;

    let packages = vec![
        "vim".to_string(),
        "git".to_string(),
        "curl".to_string(),
        "wget".to_string(),
        "hello".to_string(),
        "ripgrep".to_string(),
        "fd".to_string(),
        "bat".to_string(),
        "jq".to_string(),
        "htop".to_string(),
    ];

    let start = Instant::now();
    let result = validate_packages(&packages, None);
    let duration = start.elapsed();

    // Should complete in less than 10 seconds (being generous for CI)
    assert!(
        duration.as_secs() < 10,
        "Validation took too long: {:?}",
        duration
    );

    // All packages should be valid
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.valid_packages.len(), 10);
}

#[test]
fn test_error_reporting_format() {
    let packages = vec![
        "hello".to_string(),
        "nonexistent-pkg".to_string(),
        "vim".to_string(),
    ];
    let result = validate_packages(&packages, None);

    let output = format_validation_result(&result);

    // Output should contain error type
    assert!(output.contains("PACKAGE_NOT_FOUND"));
    // Output should contain the invalid package name
    assert!(output.contains("nonexistent-pkg"));
    // Output should contain helpful link
    assert!(output.contains("https://search.nixos.org/packages"));
}

#[test]
fn test_success_message_format() {
    let packages = vec!["hello".to_string(), "vim".to_string()];
    let result = validate_packages(&packages, None);

    let output = format_validation_result(&result);

    // Success message should be present
    assert!(output.contains("✓"));
    assert!(output.contains("successfully"));
    assert!(output.contains("2 package"));
}
