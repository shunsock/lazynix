use predicates::prelude::*;

mod common;
use common::*;

#[test]
fn test_generate_help_message() {
    lnix_cmd()
        .arg("generate")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Generate flake.nix"));
}

#[test]
fn test_generate_missing_config_file() {
    let temp_dir = setup_test_dir();

    lnix_cmd()
        .arg("-C")
        .arg(temp_dir.path())
        .arg("generate")
        .assert()
        .failure()
        .stderr(predicate::str::contains("lazynix.yaml"));
}

#[test]
fn test_generate_invalid_yaml_syntax() {
    let temp_dir = setup_test_dir_with_config(&invalid_yaml_config());

    lnix_cmd()
        .arg("-C")
        .arg(temp_dir.path())
        .arg("generate")
        .assert()
        .failure();
}

#[test]
fn test_generate_writes_flake_nix_without_nix_subprocess() {
    let temp_dir = setup_test_dir_with_config(&config_with_packages(&["bash"], &[]));

    lnix_cmd()
        .env("PATH", "")
        .arg("-C")
        .arg(temp_dir.path())
        .arg("generate")
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success();

    assert!(temp_dir.path().join("flake.nix").exists());
}
