//! Integration tests for the linter functionality

use lnix_domain::PackageName;
use lnix_linter::{ValidationError, format_validation_result, validate_packages};

fn packages(names: &[&str]) -> Vec<PackageName> {
    names.iter().map(|name| name.parse().unwrap()).collect()
}

#[test]
fn test_all_valid_packages() {
    // Arrange
    let targets = packages(&["hello", "vim", "git"]);

    // Act
    let result = validate_packages(&targets, None);

    // Assert
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.valid_packages.len(), 3);
    assert!(result.valid_packages.contains(&"hello".to_string()));
    assert!(result.valid_packages.contains(&"vim".to_string()));
    assert!(result.valid_packages.contains(&"git".to_string()));
}

#[test]
fn test_non_existent_packages() {
    // Arrange
    let targets = packages(&["nonexistent-pkg-xyz-12345", "another-fake-package-99999"]);

    // Act
    let result = validate_packages(&targets, None);

    // Assert
    assert_eq!(result.valid_packages.len(), 0);
    assert_eq!(result.errors.len(), 2);
    for error in &result.errors {
        assert!(matches!(error, ValidationError::PackageNotFound { .. }));
    }
}

#[test]
fn test_mixed_valid_invalid_packages() {
    // Arrange
    let targets = packages(&["hello", "nonexistent-xyz", "vim"]);

    // Act
    let result = validate_packages(&targets, None);

    // Assert
    assert_eq!(result.valid_packages.len(), 2);
    assert_eq!(result.errors.len(), 1);
    assert!(result.valid_packages.contains(&"hello".to_string()));
    assert!(result.valid_packages.contains(&"vim".to_string()));
}

#[test]
fn test_performance_10_packages() {
    use std::time::Instant;

    // Arrange
    let targets = packages(&[
        "vim", "git", "curl", "wget", "hello", "ripgrep", "fd", "bat", "jq", "htop",
    ]);

    // Act
    let start = Instant::now();
    let result = validate_packages(&targets, None);
    let duration = start.elapsed();

    // Assert: parallel validation should finish well within 10 seconds
    assert!(
        duration.as_secs() < 10,
        "Validation took too long: {:?}",
        duration
    );
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.valid_packages.len(), 10);
}

#[test]
fn test_error_reporting_format() {
    // Arrange
    let targets = packages(&["hello", "nonexistent-pkg", "vim"]);

    // Act
    let result = validate_packages(&targets, None);
    let output = format_validation_result(&result);

    // Assert
    assert!(output.contains("PACKAGE_NOT_FOUND"));
    assert!(output.contains("nonexistent-pkg"));
    assert!(output.contains("https://search.nixos.org/packages"));
}

#[test]
fn test_success_message_format() {
    // Arrange
    let targets = packages(&["hello", "vim"]);

    // Act
    let result = validate_packages(&targets, None);
    let output = format_validation_result(&result);

    // Assert
    assert!(output.contains("✓"));
    assert!(output.contains("successfully"));
    assert!(output.contains("2 package"));
}
