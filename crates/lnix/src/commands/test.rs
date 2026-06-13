//! `lnix test` — generate the flake and run declared test commands.

use std::path::Path;
use std::process;

use lnix_nix_dispatcher::run_nix_test;

use crate::commands::pipeline::{load_config, maybe_update_lock, write_flake};
use crate::error::{LazyNixError, Result};

/// Renders `flake.nix` and runs the test commands in `nix develop`.
///
/// Fails fast (before writing the flake) when no tests are declared.
pub fn execute(config_dir: &Path, update_lock: bool) -> Result<()> {
    let loaded = load_config(config_dir)?;

    if loaded.config.dev_shell.test.is_empty() {
        return Err(LazyNixError::Validation(
            "No test commands defined in lazynix.yaml. Add test attribute to devShell.".to_string(),
        ));
    }

    write_flake(&loaded)?;
    maybe_update_lock(update_lock)?;

    println!();
    let exit_code = run_nix_test()?;
    if exit_code != 0 {
        process::exit(exit_code);
    }
    Ok(())
}
