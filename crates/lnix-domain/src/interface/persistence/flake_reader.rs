//! Port for reading pinned-package resolutions from an existing `flake.nix`.

use std::collections::HashMap;

use crate::error::FlakeError;
use crate::values::{CommitSha, PackageName, PackageVersion};

/// The nixpkgs commit and attribute path a `(name, version)` request
/// resolved to last time the flake was rendered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinnedResolution {
    pub commit: CommitSha,
    pub attr: String,
}

/// Map of pinned-package requests to their recovered resolutions.
pub type PinnedResolutions = HashMap<(PackageName, PackageVersion), PinnedResolution>;

/// Reads pinned-package resolutions from an existing `flake.nix`.
///
/// The `flake.nix` LazyNix generates is the source of truth for the
/// `(nixpkgs commit, attribute)` pair that a `(name, version)` request
/// resolved to. Callers use this port to reuse a prior resolution and
/// avoid re-invoking `nix-versions`, which would otherwise require a
/// subprocess and network I/O and would return non-deterministic
/// results whenever the underlying channel moves.
///
/// A missing or malformed file is not an error: the implementation
/// returns an empty map (or drops the malformed entry) and the caller
/// falls back to the resolver.
pub trait FlakeReader {
    /// Returns a map from `(name, version)` to its [`PinnedResolution`],
    /// restricted to the `pinned_keys` supplied by the caller.
    ///
    /// `pinned_keys` is the pinned `(name, version)` list from
    /// `lazynix.yaml`. The implementation re-encodes each key with the
    /// same naming scheme the writer uses and looks for the matching
    /// lines in `flake.nix`, rather than decoding the flake's own
    /// names; only entries whose input line and `buildInputs` line are
    /// both present are included, and partially-recovered entries are
    /// dropped so the caller re-resolves them from scratch.
    ///
    /// PRECONDITION: `pinned_keys` must have already passed
    /// `validate_config`, i.e. no two keys may encode to a colliding
    /// input or binding name.
    fn read_pinned_inputs(
        &self,
        pinned_keys: &[(PackageName, PackageVersion)],
    ) -> Result<PinnedResolutions, FlakeError>;
}
