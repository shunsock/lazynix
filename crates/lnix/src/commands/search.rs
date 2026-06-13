//! `lnix search` — discover available package versions via nix-versions.

use lnix_core::PackageName;
use lnix_nix_dispatcher::search_versions;

use crate::error::Result;

/// Searches for versions of `package_name`, optionally constrained by
/// `version`, and prints the nix-versions output.
pub fn execute(package_name: &str, version: Option<&str>, json: bool, one: bool) -> Result<()> {
    // Validate the name before forwarding it to nix-versions
    let package_name: PackageName = package_name.parse()?;

    let output = search_versions(package_name.as_str(), version, json, one)?;
    print!("{}", output);
    Ok(())
}
