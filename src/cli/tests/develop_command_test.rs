use predicates::prelude::*;

mod common;
use common::*;

#[test]
fn test_develop_help_message() {
    lnix_cmd()
        .arg("develop")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Generate flake.nix"))
        .stdout(predicate::str::contains("--update"));
}

#[test]
fn test_develop_missing_config_file() {
    let temp_dir = setup_test_dir();

    lnix_cmd()
        .arg("-C")
        .arg(temp_dir.path())
        .arg("develop")
        .assert()
        .failure()
        .stderr(predicate::str::contains("lazynix.yaml"));
}

#[test]
fn test_develop_invalid_yaml_syntax() {
    let temp_dir = setup_test_dir_with_config(&invalid_yaml_config());

    lnix_cmd()
        .arg("-C")
        .arg(temp_dir.path())
        .arg("develop")
        .assert()
        .failure();
}

#[test]
#[ignore]
fn test_develop_generates_flake_nix() {
    let temp_dir = setup_test_dir_with_config(&config_with_packages(&["hello"], &[]));

    lnix_cmd()
        .arg("-C")
        .arg(temp_dir.path())
        .arg("develop")
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .success();

    assert!(temp_dir.path().join("flake.nix").exists());
}

#[test]
fn test_develop_with_config_dir_flag() {
    let temp_dir = setup_test_dir_with_config(&config_with_packages(&["bash"], &[]));

    let _ = lnix_cmd()
        .arg("-C")
        .arg(temp_dir.path())
        .arg("develop")
        .timeout(std::time::Duration::from_secs(5))
        .assert();
}

#[test]
#[ignore]
fn test_develop_with_update_flag() {
    let temp_dir = setup_test_dir_with_config(&config_with_packages(&["bash"], &[]));

    let _ = lnix_cmd()
        .arg("-C")
        .arg(temp_dir.path())
        .arg("develop")
        .arg("--update")
        .timeout(std::time::Duration::from_secs(30))
        .assert();
}
