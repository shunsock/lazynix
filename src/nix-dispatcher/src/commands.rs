use std::process::{Command, Stdio};

use crate::error::{NixDispatcherError, Result};

pub fn run_nix_develop() -> Result<()> {
    println!("Entering nix develop shell...");

    let status = Command::new("nix")
        .arg("develop")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| {
            NixDispatcherError::CommandExecution(format!("Failed to execute 'nix develop': {}", e))
        })?;

    if !status.success() {
        return Err(NixDispatcherError::CommandExecution(format!(
            "nix develop exited with status: {}",
            status
        )));
    }

    Ok(())
}

pub fn run_flake_update() -> Result<()> {
    println!("Updating flake.lock...");

    let output = Command::new("nix")
        .arg("flake")
        .arg("update")
        .output()
        .map_err(|e| {
            NixDispatcherError::CommandExecution(format!(
                "Failed to execute 'nix flake update': {}",
                e
            ))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(NixDispatcherError::CommandExecution(format!(
            "nix flake update failed: {}",
            stderr
        )));
    }

    println!("flake.lock updated successfully");
    Ok(())
}

pub fn run_nix_develop_command(cmd_args: Vec<String>) -> Result<i32> {
    let mut cmd = Command::new("nix");
    cmd.arg("develop").arg("-c");

    for arg in cmd_args {
        cmd.arg(arg);
    }

    let status = cmd
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| {
            NixDispatcherError::CommandExecution(format!(
                "Failed to execute 'nix develop -c': {}",
                e
            ))
        })?;

    let exit_code = status.code().unwrap_or(-1);
    Ok(exit_code)
}

pub fn run_nix_test() -> Result<i32> {
    println!("Running tests in nix develop environment...");

    let mut cmd = Command::new("nix");
    cmd.arg("develop")
        .arg("-c")
        .arg("bash")
        .arg("-c")
        .arg("true")
        .env("LAZYNIX_TEST_MODE", "1");

    let status = cmd
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| {
            NixDispatcherError::CommandExecution(format!(
                "Failed to execute 'nix develop' for tests: {}",
                e
            ))
        })?;

    let exit_code = status.code().unwrap_or(-1);
    Ok(exit_code)
}

pub fn run_task_in_nix_env(commands: Vec<String>) -> Result<i32> {
    // Join multiple commands with && for sequential execution
    let script = commands.join(" && ");

    let mut cmd = Command::new("nix");
    cmd.arg("develop")
        .arg("-c")
        .arg("sh")
        .arg("-c")
        .arg(&script);

    let status = cmd
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| {
            NixDispatcherError::CommandExecution(format!(
                "Failed to execute task in nix environment: {}",
                e
            ))
        })?;

    let exit_code = status.code().unwrap_or(-1);
    Ok(exit_code)
}
