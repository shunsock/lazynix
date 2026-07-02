//! Port for resolving package versions via the nix-versions registry.

use crate::error::NixError;
use crate::values::{PackageName, PackageVersion};

/// A pinned package resolved to a concrete nixpkgs commit and attribute.
#[derive(Debug, Clone)]
pub struct ResolvedVersion {
    /// The nixpkgs commit that provides the requested version.
    pub commit: String,
    /// The package attribute at that commit (e.g. `go_1_21`).
    pub attr: String,
}

/// Resolves and searches package versions (backed by nix-versions).
pub trait VersionResolver {
    /// Resolves `name@version` to a concrete commit and attribute.
    fn resolve(
        &self,
        name: &PackageName,
        version: &PackageVersion,
    ) -> Result<ResolvedVersion, NixError>;

    /// Searches available versions and returns the raw registry output
    /// (text or JSON as requested) for passthrough display.
    fn search(
        &self,
        name: &PackageName,
        version_constraint: Option<&str>,
        json: bool,
        one: bool,
    ) -> Result<String, NixError>;
}
