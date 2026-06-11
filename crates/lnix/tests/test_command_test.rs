use predicates::prelude::*;

mod common;
use common::*;

#[test]
fn test_test_help_message() {
    lnix_cmd()
        .arg("test")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Run tests"))
        .stdout(predicate::str::contains("--update"));
}

#[test]
fn test_test_missing_config_file() {
    let temp_dir = setup_test_dir();

    lnix_cmd()
        .arg("-C")
        .arg(temp_dir.path())
        .arg("test")
        .assert()
        .failure()
        .stderr(predicate::str::contains("lazynix.yaml"));
}

#[test]
fn test_test_no_test_commands_defined() {
    let temp_dir = setup_test_dir_with_config(&config_with_packages(&["bash"], &[]));

    lnix_cmd()
        .arg("-C")
        .arg(temp_dir.path())
        .arg("test")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("No tests defined")
                .or(predicate::str::contains("test commands")),
        );
}

#[test]
#[ignore]
fn test_test_generates_flake_before_running() {
    let temp_dir = setup_test_dir_with_config(&config_with_test_commands(&["echo test"]));

    lnix_cmd()
        .arg("-C")
        .arg(temp_dir.path())
        .arg("test")
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .success();

    assert!(temp_dir.path().join("flake.nix").exists());
}

#[test]
fn test_test_invalid_yaml() {
    let temp_dir = setup_test_dir_with_config(&invalid_yaml_config());

    lnix_cmd()
        .arg("-C")
        .arg(temp_dir.path())
        .arg("test")
        .assert()
        .failure();
}

#[test]
#[ignore]
fn test_test_with_update_flag() {
    let temp_dir = setup_test_dir_with_config(&config_with_test_commands(&["echo test"]));

    let _ = lnix_cmd()
        .arg("-C")
        .arg(temp_dir.path())
        .arg("test")
        .arg("--update")
        .timeout(std::time::Duration::from_secs(30))
        .assert();
}
