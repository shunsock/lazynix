//! Subprocess-backed [`NixRunner`] (interactive `nix` invocations).

use std::process::Command;

use lnix_domain::NixError;
use lnix_domain::interface::gateway::NixRunner;

use crate::process::run_inherit;

/// Runs `nix` with inherited stdio via [`run_inherit`].
pub struct SubprocessNixRunner;

fn nix() -> Command {
    Command::new("nix")
}

impl NixRunner for SubprocessNixRunner {
    fn develop(&self) -> Result<(), NixError> {
        let mut command = nix();
        command.arg("develop");
        match run_inherit(command)? {
            0 => Ok(()),
            code => Err(NixError::NonZeroExit(code)),
        }
    }

    fn develop_command(&self, args: &[String]) -> Result<i32, NixError> {
        let mut command = nix();
        command.arg("develop").arg("-c").args(args);
        run_inherit(command)
    }

    fn test(&self) -> Result<i32, NixError> {
        // LAZYNIX_TEST_MODE makes the generated shellHook run the
        // declared test commands instead of opening a shell.
        let mut command = nix();
        command
            .arg("develop")
            .arg("-c")
            .arg("bash")
            .arg("-c")
            .arg("true")
            .env("LAZYNIX_TEST_MODE", "1");
        run_inherit(command)
    }

    fn run_task(&self, commands: &[String]) -> Result<i32, NixError> {
        let script = commands.join(" && ");
        let mut command = nix();
        command
            .arg("develop")
            .arg("-c")
            .arg("sh")
            .arg("-c")
            .arg(&script);
        run_inherit(command)
    }

    fn flake_update(&self) -> Result<(), NixError> {
        let mut command = nix();
        command.arg("flake").arg("update");
        match run_inherit(command)? {
            0 => Ok(()),
            code => Err(NixError::NonZeroExit(code)),
        }
    }
}
