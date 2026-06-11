use predicates::prelude::*;
use std::fs;

mod common;
use common::*;

#[test]
fn test_config_dir_flag_short() {
    let temp_dir = setup_test_dir();
    let config_dir = temp_dir.path().join("config");
    fs::create_dir(&config_dir).expect("Failed to create config dir");
    fs::write(
        config_dir.join("lazynix.yaml"),
        config_with_packages(&["hello"], &[]),
    )
    .expect("Failed to write config");

    lnix_cmd()
        .arg("-C")
        .arg(&config_dir)
        .arg("lint")
        .assert()
        .success();
}

#[test]
fn test_config_dir_flag_long() {
    let temp_dir = setup_test_dir();
    let config_dir = temp_dir.path().join("config");
    fs::create_dir(&config_dir).expect("Failed to create config dir");
    fs::write(
        config_dir.join("lazynix.yaml"),
        config_with_packages(&["hello"], &[]),
    )
    .expect("Failed to write config");

    lnix_cmd()
        .arg("--config-dir")
        .arg(&config_dir)
        .arg("lint")
        .assert()
        .success();
}

#[test]
fn test_config_dir_env_variable() {
    let temp_dir = setup_test_dir_with_config(&config_with_packages(&["hello"], &[]));

    lnix_cmd()
        .env("LAZYNIX_CONFIG_DIR", temp_dir.path())
        .arg("lint")
        .assert()
        .success();
}

#[test]
fn test_config_dir_default_current_directory() {
    let temp_dir = setup_test_dir_with_config(&config_with_packages(&["hello"], &[]));

    lnix_cmd()
        .current_dir(temp_dir.path())
        .arg("lint")
        .assert()
        .success();
}

#[test]
fn test_config_dir_nonexistent_directory() {
    let temp_dir = setup_test_dir();
    let nonexistent = temp_dir.path().join("nonexistent");

    lnix_cmd()
        .arg("-C")
        .arg(nonexistent)
        .arg("lint")
        .assert()
        .failure()
        .stderr(predicate::str::contains("lazynix.yaml"));
}

#[test]
fn test_config_dir_with_spaces_in_path() {
    let temp_dir = setup_test_dir();
    let config_dir = temp_dir.path().join("config with spaces");
    fs::create_dir(&config_dir).expect("Failed to create config dir");
    fs::write(
        config_dir.join("lazynix.yaml"),
        config_with_packages(&["hello"], &[]),
    )
    .expect("Failed to write config");

    lnix_cmd()
        .arg("-C")
        .arg(&config_dir)
        .arg("lint")
        .assert()
        .success();
}
