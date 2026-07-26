//! Port for reading pinned-package resolutions from an existing `flake.nix`.

use std::collections::HashMap;

use crate::error::FlakeError;
use crate::values::{PackageName, PackageVersion};

/// The `(commit, attr)` pair a `(name, version)` request resolved to
/// last time the flake was rendered.
pub type PinnedResolution = (String, String);

/// Map of pinned-package requests to their recovered resolutions.
pub type PinnedResolutions = HashMap<(PackageName, PackageVersion), PinnedResolution>;

/// Reads pinned-package resolutions from an existing `flake.nix`.
///
/// The `flake.nix` LazyNix generates is the source of truth for the
/// `(nixpkgs commit, attribute)` pair that a `(name, version)` request
/// resolved to. Callers use this port to reuse a prior resolution and
/// avoid re-invoking `nix-versions`.
///
/// A missing or malformed file is not an error: the implementation
/// returns an empty map (or drops the malformed entry) and the caller
/// falls back to the resolver.
pub trait FlakeReader {
    /// Returns a map from `(name, version)` to `(commit, attr)`.
    ///
    /// Only entries whose input line and `buildInputs` line are both
    /// present in the flake are included; partially-recovered entries
    /// are dropped so the caller re-resolves them from scratch.
    fn read_pinned_inputs(&self) -> Result<PinnedResolutions, FlakeError>;
}
