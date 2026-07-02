//! Shared subprocess execution for the gateway adapters.
//!
//! Two helpers, one per execution shape, so stdio wiring and
//! error mapping live in exactly one place:
//!
//! - [`run_inherit`] — interactive commands; the child owns the
//!   terminal and only the exit code comes back.
//! - [`run_capture`] — capturing commands; stdout/stderr come back for
//!   the caller to interpret.

use std::process::{Command, Stdio};

use lnix_domain::NixError;

/// Captured output of a completed subprocess.
#[derive(Debug)]
pub(crate) struct Captured {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

/// Runs `command` with inherited stdio and returns its exit code.
pub(crate) fn run_inherit(mut command: Command) -> Result<i32, NixError> {
    let status = command
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    status.code().ok_or(NixError::NoExitCode)
}

/// Runs `command` capturing stdout/stderr.
///
/// A non-zero exit is NOT an error here: capturing callers (eval,
/// version resolution) interpret failure output themselves.
pub(crate) fn run_capture(mut command: Command) -> Result<Captured, NixError> {
    let output = command.output()?;
    Ok(Captured {
        success: output.status.success(),
        stdout: String::from_utf8(output.stdout)?,
        stderr: String::from_utf8(output.stderr)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inherit_returns_exit_code_of_failing_command() {
        // Arrange
        let mut command = Command::new("sh");
        command.arg("-c").arg("exit 3");

        // Act
        let code = run_inherit(command).unwrap();

        // Assert
        assert_eq!(code, 3);
    }

    #[test]
    fn capture_collects_stdout_and_stderr() {
        // Arrange
        let mut command = Command::new("sh");
        command.arg("-c").arg("echo out; echo err 1>&2");

        // Act
        let captured = run_capture(command).unwrap();

        // Assert
        assert!(captured.success);
        assert_eq!(captured.stdout, "out\n");
        assert_eq!(captured.stderr, "err\n");
    }

    #[test]
    fn capture_reports_failure_without_erroring() {
        // Arrange
        let mut command = Command::new("sh");
        command.arg("-c").arg("echo boom 1>&2; exit 1");

        // Act
        let captured = run_capture(command).unwrap();

        // Assert
        assert!(!captured.success);
        assert_eq!(captured.stderr, "boom\n");
    }

    #[test]
    fn spawn_failure_maps_to_nix_error() {
        // Arrange
        let command = Command::new("definitely-not-a-real-binary-xyz");

        // Act
        let result = run_capture(command);

        // Assert
        assert!(matches!(result, Err(NixError::Spawn(_))));
    }
}
