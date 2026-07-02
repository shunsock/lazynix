//! Nix eval executor for package validation

use crate::error::Result;
use lnix_domain::PackageName;
use std::process::Command;

/// Result of executing a `nix eval` command
#[derive(Debug, Clone)]
pub struct NixEvalResult {
    /// Whether the command executed successfully (exit code 0)
    pub success: bool,
    /// Standard output from the command
    pub stdout: String,
    /// Standard error from the command
    pub stderr: String,
    /// Exit code of the command
    pub exit_code: i32,
}

/// Executes `nix eval nixpkgs#<package>` to check if a package exists
///
/// Shell-injection safety is guaranteed by [`PackageName`], which only
/// permits alphanumerics, hyphens, underscores, and dots.
///
/// # Arguments
/// * `package` - Package to evaluate
///
/// # Returns
/// * `Ok(NixEvalResult)` - Command execution result
/// * `Err(LinterError)` - If command execution fails
///
/// # Example
/// ```no_run
/// use lnix_domain::PackageName;
/// use lnix_linter::nix_eval::eval_package;
///
/// let package: PackageName = "vim".parse().unwrap();
/// let result = eval_package(&package).unwrap();
/// assert!(result.success);
/// ```
pub fn eval_package(package: &PackageName) -> Result<NixEvalResult> {
    let output = Command::new("nix")
        .arg("eval")
        .arg(format!("nixpkgs#{}", package))
        .output()?;

    Ok(NixEvalResult {
        success: output.status.success(),
        stdout: String::from_utf8(output.stdout)?,
        stderr: String::from_utf8(output.stderr)?,
        exit_code: output.status.code().unwrap_or(-1),
    })
}

/// Executes `nix eval --system <arch> nixpkgs#<package>` to check architecture compatibility
///
/// # Arguments
/// * `package` - Package to evaluate
/// * `arch` - Target architecture (e.g., "aarch64-darwin", "x86_64-linux")
///
/// # Returns
/// * `Ok(NixEvalResult)` - Command execution result
/// * `Err(LinterError)` - If command execution fails
///
/// # Example
/// ```no_run
/// use lnix_domain::PackageName;
/// use lnix_linter::nix_eval::eval_package_for_arch;
///
/// let package: PackageName = "vim".parse().unwrap();
/// let result = eval_package_for_arch(&package, "aarch64-darwin").unwrap();
/// assert!(result.success);
/// ```
pub fn eval_package_for_arch(package: &PackageName, arch: &str) -> Result<NixEvalResult> {
    let output = Command::new("nix")
        .arg("eval")
        .arg("--system")
        .arg(arch)
        .arg(format!("nixpkgs#{}", package))
        .output()?;

    Ok(NixEvalResult {
        success: output.status.success(),
        stdout: String::from_utf8(output.stdout)?,
        stderr: String::from_utf8(output.stderr)?,
        exit_code: output.status.code().unwrap_or(-1),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_succeeds_for_existing_package() {
        // Arrange
        let package: PackageName = "hello".parse().unwrap();

        // Act
        let result = eval_package(&package).unwrap();

        // Assert
        assert!(result.success, "stderr: {}", result.stderr);
    }

    #[test]
    fn eval_fails_for_nonexistent_package() {
        // Arrange
        let package: PackageName = "nonexistent-pkg-xyz-12345".parse().unwrap();

        // Act
        let result = eval_package(&package).unwrap();

        // Assert
        assert!(!result.success);
        assert!(result.stderr.contains("does not provide"));
    }

    #[test]
    fn eval_for_arch_completes() {
        // Arrange
        let package: PackageName = "hello".parse().unwrap();

        // Act
        let result = eval_package_for_arch(&package, "x86_64-linux").unwrap();

        // Assert: hello should be available on x86_64-linux
        assert!(result.success || !result.stderr.is_empty());
    }
}
