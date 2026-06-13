//! `lnix update` — update `flake.lock` without entering the shell.

use lnix_nix_dispatcher::run_flake_update;

use crate::error::Result;

/// Runs `nix flake update`.
pub fn execute() -> Result<()> {
    run_flake_update()?;
    Ok(())
}
