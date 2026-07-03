//! Port for scaffolding a new project from bundled templates.

use crate::error::{ConfigError, FlakeError};

/// Writes the bundled starter files for `lnix init`.
///
/// Templates are raw text with comments, so they bypass the typed
/// [`crate::DevShellDefinition`] round-trip on purpose: serializing a parsed
/// `DevShellDefinition` back out would strip the guidance comments users edit.
pub trait ProjectScaffolder {
    /// Whether a `lazynix.yaml` already exists at the target location.
    fn config_exists(&self) -> bool;

    /// Whether a `flake.nix` already exists at the target location.
    fn flake_exists(&self) -> bool;

    /// Display path of the config file, for user-facing messages.
    fn config_path_display(&self) -> String;

    /// Display path of the flake file, for user-facing messages.
    fn flake_path_display(&self) -> String;

    /// Writes the starter `lazynix.yaml`, replacing any existing file.
    fn write_config_template(&self) -> Result<(), ConfigError>;

    /// Writes the starter `flake.nix`, replacing any existing file.
    fn write_flake_template(&self) -> Result<(), FlakeError>;
}
