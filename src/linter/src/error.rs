//! Error types for the LazyNix Linter

use std::io;
use thiserror::Error;

/// Errors that can occur during linting operations
#[derive(Error, Debug)]
pub enum LinterError {
    /// Failed to execute nix command
    #[error("Failed to execute nix command: {0}")]
    CommandExecutionFailed(#[from] io::Error),

    /// Invalid package name (potential shell injection)
    #[error("Invalid package name: {0}")]
    InvalidPackageName(String),

    /// Command timed out
    #[error("Nix eval command timed out after {0} seconds")]
    Timeout(u64),

    /// UTF-8 conversion error
    #[error("Failed to convert command output to UTF-8: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

/// Result type for linter operations
pub type Result<T> = std::result::Result<T, LinterError>;

/// Validation errors that occur when checking package availability
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
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

    /// Unknown error occurred during validation
    #[error("Unknown error for package '{package}': {message}")]
    UnknownError {
        /// Name of the package
        package: String,
        /// Error message
        message: String,
    },
}
