//! `lnix develop` — generate the flake and enter the dev shell.

use std::path::Path;

use lnix_nix_dispatcher::run_nix_develop;

use crate::commands::pipeline::{load_config, maybe_update_lock, write_flake};
use crate::error::Result;

/// Renders `flake.nix`, optionally updates the lock, and enters
/// `nix develop`.
pub fn execute(config_dir: &Path, update_lock: bool) -> Result<()> {
    let loaded = load_config(config_dir)?;
    write_flake(&loaded)?;
    maybe_update_lock(update_lock)?;

    println!();
    run_nix_develop()?;
    Ok(())
}
