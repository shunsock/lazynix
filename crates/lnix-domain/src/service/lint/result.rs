//! Aggregate result of validating a set of packages.

use super::error::PackageValidationError;

/// Result of validating multiple packages
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Packages that passed validation
    pub valid_packages: Vec<String>,
    /// Validation errors encountered
    pub errors: Vec<PackageValidationError>,
}
