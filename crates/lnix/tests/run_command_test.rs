use predicates::prelude::*;

mod common;
use common::*;

#[test]
fn test_run_help_message() {
    lnix_cmd()
        .arg("run")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Run a command"))
        .stdout(predicate::str::contains("--update"))
        .stdout(predicate::str::contains("--no-regen"));
}

#[test]
fn test_run_missing_config_file() {
    let temp_dir = setup_test_dir();

    lnix_cmd()
        .arg("-C")
        .arg(temp_dir.path())
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .failure()
        .stderr(predicate::str::contains("lazynix.yaml"));
}

#[test]
fn test_run_empty_command_fails() {
    let temp_dir = setup_test_dir_with_config(&config_with_packages(&["bash"], &[]));

    lnix_cmd()
        .arg("-C")
        .arg(temp_dir.path())
        .arg("run")
        .assert()
        .failure();
}

#[test]
fn test_run_with_no_regen_flag() {
    let temp_dir = setup_test_dir_with_config(&config_with_packages(&["bash"], &[]));

    let _ = lnix_cmd()
        .arg("-C")
        .arg(temp_dir.path())
        .arg("run")
        .arg("--no-regen")
        .arg("echo")
        .arg("test")
        .timeout(std::time::Duration::from_secs(5))
        .assert();
}

#[test]
fn test_run_with_update_flag() {
    let temp_dir = setup_test_dir_with_config(&config_with_packages(&["bash"], &[]));

    let _ = lnix_cmd()
        .arg("-C")
        .arg(temp_dir.path())
        .arg("run")
        .arg("--update")
        .arg("echo")
        .arg("test")
        .timeout(std::time::Duration::from_secs(5))
        .assert();
}

#[test]
fn test_run_with_trailing_arguments() {
    let temp_dir = setup_test_dir_with_config(&config_with_packages(&["bash"], &[]));

    let _ = lnix_cmd()
        .arg("-C")
        .arg(temp_dir.path())
        .arg("run")
        .arg("echo")
        .arg("hello")
        .arg("world")
        .timeout(std::time::Duration::from_secs(5))
        .assert();
}

#[test]
fn test_run_with_hyphen_arguments() {
    let temp_dir = setup_test_dir_with_config(&config_with_packages(&["bash"], &[]));

    let _ = lnix_cmd()
        .arg("-C")
        .arg(temp_dir.path())
        .arg("run")
        .arg("echo")
        .arg("--")
        .arg("--test-flag")
        .timeout(std::time::Duration::from_secs(5))
        .assert();
}

#[test]
#[ignore]
fn test_run_executes_command_in_nix_env() {
    let temp_dir = setup_test_dir_with_config(&config_with_packages(&["hello"], &[]));

    lnix_cmd()
        .arg("-C")
        .arg(temp_dir.path())
        .arg("run")
        .arg("hello")
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello, world!"));
}
