//! Port for capturing `nix eval` outcomes.

use crate::error::NixError;
use crate::values::PackageName;

/// Captured outcome of evaluating one package with `nix eval`.
///
/// Only what [`crate::service::lint`] needs to classify the result:
/// success, and stderr for pattern matching on failure.
#[derive(Debug, Clone)]
pub struct EvalOutcome {
    /// Whether `nix eval` exited successfully.
    pub success: bool,
    /// Standard error output, used to classify failures.
    pub stderr: String,
}

/// Evaluates package availability via `nix eval`, capturing output.
///
/// Taking [`PackageName`] (not `&str`) carries the shell-injection
/// guard of the value object across the process boundary.
pub trait NixEvaluator {
    /// Evaluates `nixpkgs#<package>`, optionally against an explicit
    /// target architecture (e.g. `aarch64-darwin`).
    fn eval_package(
        &self,
        package: &PackageName,
        arch: Option<&str>,
    ) -> Result<EvalOutcome, NixError>;
}
