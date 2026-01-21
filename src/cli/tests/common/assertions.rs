use predicates::BoxPredicate;
use predicates::prelude::*;

/// Exit code constants
#[allow(dead_code)]
pub const EXIT_SUCCESS: i32 = 0;
#[allow(dead_code)]
pub const EXIT_FAILURE: i32 = 1;

/// Predicate for success messages
#[allow(dead_code)]
pub fn contains_success_message() -> BoxPredicate<str> {
    predicate::str::contains("✓")
        .or(predicate::str::contains("successfully"))
        .boxed()
}

/// Predicate for package validation errors
#[allow(dead_code)]
pub fn contains_package_validation_error(package: &str) -> BoxPredicate<str> {
    predicate::str::contains("PACKAGE_NOT_FOUND")
        .and(predicate::str::contains(package))
        .boxed()
}

/// Predicate for valid YAML content
#[allow(dead_code)]
pub fn is_valid_yaml() -> BoxPredicate<str> {
    predicate::function(|s: &str| serde_yaml::from_str::<serde_yaml::Value>(s).is_ok()).boxed()
}
