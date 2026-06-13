//! `lnix run` — run a command inside the dev environment.

use std::path::Path;

use lnix_nix_dispatcher::run_nix_develop_command;

use crate::commands::pipeline::{load_config, maybe_update_lock, write_flake};
use crate::error::{LazyNixError, Result};

/// Renders `flake.nix` (unless `--no-regen`) and runs `cmd_args` in
/// `nix develop`. Returns the command's exit code.
pub fn execute(
    config_dir: &Path,
    update_lock: bool,
    regenerate: bool,
    cmd_args: Vec<String>,
) -> Result<i32> {
    if cmd_args.is_empty() {
        return Err(LazyNixError::Validation(
            "command arguments cannot be empty".to_string(),
        ));
    }

    if regenerate {
        let loaded = load_config(config_dir)?;
        write_flake(&loaded)?;
    } else {
        println!("Using existing flake.nix (--no-regen specified)");
    }

    maybe_update_lock(update_lock)?;

    println!();
    println!("Running command: {}", cmd_args.join(" "));
    let exit_code = run_nix_develop_command(cmd_args)?;
    Ok(exit_code)
}
