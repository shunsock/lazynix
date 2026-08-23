//! Port for reading the project's configuration files.

use crate::definition::{DevShellDefinition, Settings};
use crate::error::ConfigError;

/// Reads `lazynix.yaml` and `lazynix-settings.yaml`.
///
/// Implementations own the location of these files (the config
/// directory); callers never handle paths. The port is read-only: the
/// generated `flake.nix` is the source of truth for resolved pinned
/// versions, so `lazynix.yaml` is never rewritten.
pub trait ConfigRepository {
    /// Reads and deserializes `lazynix.yaml`.
    fn read_config(&self) -> Result<DevShellDefinition, ConfigError>;

    /// Reads `lazynix-settings.yaml`, or `None` when the file is absent.
    fn read_settings(&self) -> Result<Option<Settings>, ConfigError>;
}
