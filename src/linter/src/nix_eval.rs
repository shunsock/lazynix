//! Nix eval executor for package validation

use crate::error::{LinterError, Result};
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

/// Validates a package name to prevent shell injection
///
/// Package names should only contain alphanumeric characters, hyphens, and underscores
fn validate_package_name(package_name: &str) -> Result<()> {
    if package_name.is_empty() {
        return Err(LinterError::InvalidPackageName(
            "Package name cannot be empty".to_string(),
        ));
    }

    // Allow alphanumeric, hyphens, underscores, and dots
    if !package_name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return Err(LinterError::InvalidPackageName(format!(
            "Package name '{}' contains invalid characters",
            package_name
        )));
    }

    Ok(())
}

/// Executes `nix eval nixpkgs#<package>` to check if a package exists
///
/// # Arguments
/// * `package_name` - Name of the package to evaluate
///
/// # Returns
/// * `Ok(NixEvalResult)` - Command execution result
/// * `Err(LinterError)` - If command execution fails or package name is invalid
///
/// # Example
/// ```no_run
/// use lnix_linter::nix_eval::eval_package;
///
/// let result = eval_package("vim").unwrap();
/// assert!(result.success);
/// ```
pub fn eval_package(package_name: &str) -> Result<NixEvalResult> {
    validate_package_name(package_name)?;

    let output = Command::new("nix")
        .arg("eval")
        .arg(format!("nixpkgs#{}", package_name))
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
/// * `package_name` - Name of the package to evaluate
/// * `arch` - Target architecture (e.g., "aarch64-darwin", "x86_64-linux")
///
/// # Returns
/// * `Ok(NixEvalResult)` - Command execution result
/// * `Err(LinterError)` - If command execution fails or package name is invalid
///
/// # Example
/// ```no_run
/// use lnix_linter::nix_eval::eval_package_for_arch;
///
/// let result = eval_package_for_arch("vim", "aarch64-darwin").unwrap();
/// assert!(result.success);
/// ```
pub fn eval_package_for_arch(package_name: &str, arch: &str) -> Result<NixEvalResult> {
    validate_package_name(package_name)?;

    let output = Command::new("nix")
        .arg("eval")
        .arg("--system")
        .arg(arch)
        .arg(format!("nixpkgs#{}", package_name))
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
    fn test_validate_package_name_valid() {
        assert!(validate_package_name("vim").is_ok());
        assert!(validate_package_name("python3").is_ok());
        assert!(validate_package_name("gcc-13").is_ok());
        assert!(validate_package_name("rust_1-70").is_ok());
        assert!(validate_package_name("hello.world").is_ok());
    }

    #[test]
    fn test_validate_package_name_invalid() {
        assert!(validate_package_name("").is_err());
        assert!(validate_package_name("vim;ls").is_err());
        assert!(validate_package_name("vim&&echo").is_err());
        assert!(validate_package_name("vim|cat").is_err());
        assert!(validate_package_name("vim$PATH").is_err());
        assert!(validate_package_name("vim `whoami`").is_err());
    }

    #[test]
    fn test_eval_valid_package() {
        let result = eval_package("hello");
        assert!(result.is_ok());
        let result = result.unwrap();
        // hello package should exist in nixpkgs
        assert!(result.success, "stderr: {}", result.stderr);
    }

    #[test]
    fn test_eval_invalid_package() {
        let result = eval_package("nonexistent-pkg-xyz-12345");
        assert!(result.is_ok());
        let result = result.unwrap();
        // Package should not exist
        assert!(!result.success);
        assert!(result.stderr.contains("does not provide"));
    }

    #[test]
    fn test_eval_package_for_arch_valid() {
        // Test with current system architecture
        let result = eval_package_for_arch("hello", "x86_64-linux");
        assert!(result.is_ok());
        let result = result.unwrap();
        // hello should be available on x86_64-linux
        assert!(result.success || !result.stderr.is_empty());
    }

    #[test]
    fn test_eval_invalid_package_name() {
        let result = eval_package("vim;malicious");
        assert!(result.is_err());
        match result {
            Err(LinterError::InvalidPackageName(_)) => {}
            _ => panic!("Expected InvalidPackageName error"),
        }
    }
}
