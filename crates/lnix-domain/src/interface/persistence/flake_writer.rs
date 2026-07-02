//! Port for persisting rendered `flake.nix` content.

use crate::error::FlakeError;

/// Persists a rendered `flake.nix`.
///
/// The rendering itself is a pure domain service
/// ([`crate::service::flake::render_flake`]); this port only covers the
/// write to disk. Implementations own the output location.
pub trait FlakeWriter {
    /// Writes `contents` as the project's `flake.nix`, replacing any
    /// existing file.
    fn write_flake(&self, contents: &str) -> Result<(), FlakeError>;
}
