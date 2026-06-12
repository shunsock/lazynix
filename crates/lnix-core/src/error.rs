//! Error types for value object parsing and config validation.

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
