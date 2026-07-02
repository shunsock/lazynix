//! Port for reading and writing the project's configuration files.

use crate::config::{Config, Settings};
use crate::error::ConfigError;

/// Reads and writes `lazynix.yaml` and reads `lazynix-settings.yaml`.
///
/// Implementations own the location of these files (the config
/// directory); callers never handle paths.
pub trait ConfigRepository {
    /// Reads and deserializes `lazynix.yaml`.
    fn read_config(&self) -> Result<Config, ConfigError>;

    /// Serializes `config` back to `lazynix.yaml`.
    ///
    /// Used to persist resolved pinned-package versions.
    fn write_config(&self, config: &Config) -> Result<(), ConfigError>;

    /// Reads `lazynix-settings.yaml`, or `None` when the file is absent.
    fn read_settings(&self) -> Result<Option<Settings>, ConfigError>;
}
