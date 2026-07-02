//! Subprocess-backed [`NixEvaluator`] (capturing `nix eval`).

use std::process::Command;

use lnix_domain::interface::gateway::{EvalOutcome, NixEvaluator};
use lnix_domain::{NixError, PackageName};

use crate::process::run_capture;

/// Evaluates `nixpkgs#<package>` via `nix eval`, capturing output.
///
/// Shell-injection safety is carried by [`PackageName`], which only
/// permits alphanumerics, hyphens, underscores, and dots.
pub struct SubprocessNixEvaluator;

impl NixEvaluator for SubprocessNixEvaluator {
    fn eval_package(
        &self,
        package: &PackageName,
        arch: Option<&str>,
    ) -> Result<EvalOutcome, NixError> {
        let mut command = Command::new("nix");
        command.arg("eval");
        if let Some(arch) = arch {
            command.arg("--system").arg(arch);
        }
        command.arg(format!("nixpkgs#{}", package));

        let captured = run_capture(command)?;
        Ok(EvalOutcome {
            success: captured.success,
            stderr: captured.stderr,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_succeeds_for_existing_package() {
        // Arrange
        let package: PackageName = "hello".parse().unwrap();

        // Act
        let outcome = SubprocessNixEvaluator.eval_package(&package, None).unwrap();

        // Assert
        assert!(outcome.success, "stderr: {}", outcome.stderr);
    }

    #[test]
    fn eval_fails_for_nonexistent_package() {
        // Arrange
        let package: PackageName = "nonexistent-pkg-xyz-12345".parse().unwrap();

        // Act
        let outcome = SubprocessNixEvaluator.eval_package(&package, None).unwrap();

        // Assert
        assert!(!outcome.success);
        assert!(outcome.stderr.contains("does not provide"));
    }

    #[test]
    fn eval_for_explicit_arch_completes() {
        // Arrange
        let package: PackageName = "hello".parse().unwrap();

        // Act
        let outcome = SubprocessNixEvaluator
            .eval_package(&package, Some("x86_64-linux"))
            .unwrap();

        // Assert
        assert!(outcome.success || !outcome.stderr.is_empty());
    }
}
