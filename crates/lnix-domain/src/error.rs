//! Domain-owned error and diagnostic types.
//!
//! Besides parse/validation errors raised by the domain itself, this
//! module defines the focused error types returned through the ports in
//! [`crate::interface`]. Keeping them here (not in infrastructure)
//! preserves the inward-only dependency direction: adapters map their
//! concrete failures into these types.

use thiserror::Error;

/// Raised when a raw string cannot be converted into a value object.
///
/// These errors surface during YAML deserialization (via `serde(try_from)`),
/// so an invalid name never enters the system as a typed value.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    #[error(
        "Invalid package name: '{0}'. Package names should contain only alphanumeric characters, hyphens, underscores, and dots (for nested attributes like python312Packages.pip)"
    )]
    InvalidPackageName(String),

    #[error("Package version cannot be empty")]
    EmptyPackageVersion,

    #[error(
        "Invalid task name: '{0}'. Task names should contain only alphanumeric characters, hyphens, and underscores"
    )]
    InvalidTaskName(String),

    #[error(
        "Invalid environment variable name: '{0}'. Variable names must match [a-zA-Z_][a-zA-Z0-9_]*"
    )]
    InvalidEnvVarName(String),

    #[error(
        "Invalid registry URL: '{0}'. Expected format: 'github:OWNER/REPO/BRANCH' (e.g., 'github:NixOS/nixpkgs/nixos-25.06')"
    )]
    InvalidRegistryUrl(String),
}

/// Raised by [`crate::validate_config`] for constraints that span
/// multiple fields and therefore cannot be expressed by a single
/// value object.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    #[error("Task '{0}' has an empty commands list. At least one command must be specified")]
    EmptyTaskCommands(String),
}

/// Non-fatal finding from [`crate::validate_config`].
///
/// Diagnostics are returned as values so the domain stays free of I/O;
/// the caller decides how (and whether) to display them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Diagnostic {
    /// The config declares no packages at all.
    NoPackages,
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Diagnostic::NoPackages => write!(f, "No packages specified in lazynix.yaml"),
        }
    }
}

/// Failures reading or writing the project's own configuration files,
/// raised through [`crate::interface::persistence::ConfigRepository`].
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("lazynix.yaml not found in {0}")]
    NotFound(String),

    #[error("Invalid YAML syntax: {0}")]
    Parse(String),

    #[error("Config validation failed: {0}")]
    Invalid(#[from] ValidationError),

    #[error("Failed to read or write config: {0}")]
    Io(#[from] std::io::Error),

    #[error("Dotenv file not found: {0}")]
    DotenvFileNotFound(String),
}

/// Failures persisting rendered `flake.nix` content, raised through
/// [`crate::interface::persistence::FlakeWriter`].
#[derive(Error, Debug)]
pub enum FlakeError {
    #[error("Failed to write flake.nix: {0}")]
    Write(#[from] std::io::Error),
}

/// Failures executing `nix`, raised through the gateways in
/// [`crate::interface::gateway`].
#[derive(Error, Debug)]
pub enum NixError {
    #[error("Failed to execute nix command: {0}")]
    Spawn(#[from] std::io::Error),

    #[error("Nix command exited with status {0}")]
    NonZeroExit(i32),

    #[error("Nix command terminated without an exit code")]
    NoExitCode,

    #[error("Failed to convert command output to UTF-8: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),

    #[error("Version resolution failed for '{spec}': {message}")]
    VersionResolution {
        /// The `name@version` spec that was being resolved.
        spec: String,
        /// Why resolution failed (stderr or parse detail).
        message: String,
    },
}
