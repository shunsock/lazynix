//! Application-level error: the top of the two-tier error hierarchy.
//!
//! Ports return domain-owned focused errors ([`ConfigError`],
//! [`FlakeError`], [`NixError`]); use-cases lift them into
//! [`ApplicationError`] via `?` (`#[from]`), which keeps use-case
//! bodies on the railway: any failure short-circuits, and the error's
//! category stays visible in the type.

use lnix_domain::{ConfigError, FlakeError, NixError};
use thiserror::Error;

/// Union of every failure a use-case can surface.
///
/// `transparent` delegates `Display` to the focused error, so messages
/// stay specific while `main` can still branch on the category.
#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error(transparent)]
    Config(#[from] ConfigError),

    #[error(transparent)]
    Flake(#[from] FlakeError),

    #[error(transparent)]
    Nix(#[from] NixError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifts_config_error_preserving_message() {
        // Arrange
        let source = ConfigError::NotFound("/repo".to_string());

        // Act
        let lifted: ApplicationError = source.into();

        // Assert
        assert!(matches!(lifted, ApplicationError::Config(_)));
        assert_eq!(lifted.to_string(), "lazynix.yaml not found in /repo");
    }

    #[test]
    fn lifts_flake_error_preserving_message() {
        // Arrange
        let source = FlakeError::Write(std::io::Error::other("disk full"));

        // Act
        let lifted: ApplicationError = source.into();

        // Assert
        assert!(matches!(lifted, ApplicationError::Flake(_)));
        assert_eq!(lifted.to_string(), "Failed to write flake.nix: disk full");
    }

    #[test]
    fn lifts_nix_error_preserving_message() {
        // Arrange
        let source = NixError::NonZeroExit(2);

        // Act
        let lifted: ApplicationError = source.into();

        // Assert
        assert!(matches!(lifted, ApplicationError::Nix(_)));
        assert_eq!(lifted.to_string(), "Nix command exited with status 2");
    }
}
