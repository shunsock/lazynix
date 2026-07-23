//! Classification outcomes for package availability checks.

use thiserror::Error;

/// Validation errors that occur when checking package availability
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum PackageValidationError {
    /// Package does not exist in nixpkgs
    #[error("Package '{package}' not found in nixpkgs")]
    PackageNotFound {
        /// Name of the package that was not found
        package: String,
    },

    /// Package exists but is not available for the target architecture
    #[error("Package '{package}' is not available on architecture '{arch}'")]
    ArchitectureUnsupported {
        /// Name of the package
        package: String,
        /// Target architecture
        arch: String,
    },

    /// Pinned version could not be resolved via nix-versions
    #[error("Version '{version}' of package '{package}' could not be resolved: {message}")]
    VersionNotFound {
        /// Name of the package whose version failed to resolve
        package: String,
        /// The requested version specifier
        version: String,
        /// Why resolution failed (registry stderr or parse detail)
        message: String,
    },

    /// Unknown error occurred during validation
    #[error("Unknown error for package '{package}': {message}")]
    UnknownError {
        /// Name of the package
        package: String,
        /// Error message
        message: String,
    },
}
