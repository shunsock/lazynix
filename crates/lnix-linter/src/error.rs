//! Error types for the LazyNix Linter

use std::io;
use thiserror::Error;

/// Errors that can occur during linting operations
///
/// Note: invalid package names are no longer an error case here.
/// [`lnix_domain::PackageName`] makes them unrepresentable.
#[derive(Error, Debug)]
pub enum LinterError {
    /// Failed to execute nix command
    #[error("Failed to execute nix command: {0}")]
    CommandExecutionFailed(#[from] io::Error),

    /// Command timed out
    #[error("Nix eval command timed out after {0} seconds")]
    Timeout(u64),

    /// UTF-8 conversion error
    #[error("Failed to convert command output to UTF-8: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

/// Result type for linter operations
pub type Result<T> = std::result::Result<T, LinterError>;

/// Classification of package-availability failures.
///
/// The type moved to the domain layer; this alias keeps the linter's
/// public API stable until the crate is dismantled.
pub use lnix_domain::PackageValidationError as ValidationError;
