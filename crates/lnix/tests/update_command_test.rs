use predicates::prelude::*;

mod common;
use common::*;

#[test]
fn test_update_help_message() {
    lnix_cmd()
        .arg("update")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Update flake.lock"));
}

#[test]
fn test_update_with_no_flake_file() {
    let temp_dir = setup_test_dir_with_config(&config_with_packages(&["bash"], &[]));

    let _ = lnix_cmd()
        .arg("-C")
        .arg(temp_dir.path())
        .arg("update")
        .timeout(std::time::Duration::from_secs(5))
        .assert();
}

#[test]
#[ignore]
fn test_update_updates_flake_lock() {
    let temp_dir = setup_test_dir_with_config(&config_with_packages(&["bash"], &[]));

    lnix_cmd()
        .arg("-C")
        .arg(temp_dir.path())
        .arg("update")
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .success();

    assert!(temp_dir.path().join("flake.lock").exists());
}
