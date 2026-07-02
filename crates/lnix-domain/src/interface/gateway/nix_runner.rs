//! Port for interactive `nix` invocations.

use crate::error::NixError;

/// Runs `nix` commands with inherited stdio, returning exit codes.
///
/// Arguments are arbitrary shell command strings by design — they are
/// user-authored commands to run *inside* the dev shell, not values
/// with domain invariants.
pub trait NixRunner {
    /// Enters `nix develop` interactively.
    fn develop(&self) -> Result<(), NixError>;

    /// Runs `args` via `nix develop -c`, returning the command's exit code.
    fn develop_command(&self, args: &[String]) -> Result<i32, NixError>;

    /// Runs the declared test commands (via `LAZYNIX_TEST_MODE`),
    /// returning the exit code.
    fn test(&self) -> Result<i32, NixError>;

    /// Runs `commands` sequentially (joined with `&&`) inside the dev
    /// shell, returning the exit code.
    fn run_task(&self, commands: &[String]) -> Result<i32, NixError>;

    /// Runs `nix flake update`.
    fn flake_update(&self) -> Result<(), NixError>;
}
