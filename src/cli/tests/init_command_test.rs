use predicates::prelude::*;
use std::fs;

mod common;
use common::assertions::is_valid_yaml;
use common::*;

#[test]
fn test_init_creates_files_in_current_directory() {
    let temp_dir = setup_test_dir();

    lnix_cmd()
        .current_dir(temp_dir.path())
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created"));

    assert!(temp_dir.path().join("lazynix.yaml").exists());
    assert!(temp_dir.path().join("flake.nix").exists());
}

#[test]
fn test_init_creates_files_with_custom_config_dir() {
    let temp_dir = setup_test_dir();
    let config_dir = temp_dir.path().join("config");
    fs::create_dir(&config_dir).expect("Failed to create config dir");

    lnix_cmd()
        .current_dir(&config_dir)
        .arg("-C")
        .arg(&config_dir)
        .arg("init")
        .assert()
        .success();

    assert!(config_dir.join("lazynix.yaml").exists());
    assert!(config_dir.join("flake.nix").exists());
}

#[test]
fn test_init_fails_when_yaml_exists_without_force() {
    let temp_dir = setup_test_dir();
    fs::write(temp_dir.path().join("lazynix.yaml"), "existing").expect("Failed to write file");

    lnix_cmd()
        .current_dir(temp_dir.path())
        .arg("init")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("already exists"))
        .stderr(predicate::str::contains("lazynix.yaml"))
        .stderr(predicate::str::contains("--force"));
}

#[test]
fn test_init_fails_when_flake_exists_without_force() {
    let temp_dir = setup_test_dir();
    fs::write(temp_dir.path().join("flake.nix"), "existing").expect("Failed to write file");

    lnix_cmd()
        .current_dir(temp_dir.path())
        .arg("init")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("already exists"))
        .stderr(predicate::str::contains("flake.nix"))
        .stderr(predicate::str::contains("--force"));
}

#[test]
fn test_init_overwrites_with_force_flag() {
    let temp_dir = setup_test_dir();
    fs::write(temp_dir.path().join("lazynix.yaml"), "existing").expect("Failed to write file");
    fs::write(temp_dir.path().join("flake.nix"), "existing").expect("Failed to write file");

    lnix_cmd()
        .current_dir(temp_dir.path())
        .arg("init")
        .arg("--force")
        .assert()
        .success();

    let yaml_content =
        fs::read_to_string(temp_dir.path().join("lazynix.yaml")).expect("Failed to read yaml");
    assert!(yaml_content.contains("devShell"));
}

#[test]
fn test_init_fails_when_config_dir_does_not_exist() {
    let temp_dir = setup_test_dir();
    let nonexistent = temp_dir.path().join("nonexistent");

    lnix_cmd()
        .arg("-C")
        .arg(nonexistent)
        .arg("init")
        .assert()
        .failure();
}

#[test]
fn test_init_output_shows_next_steps() {
    let temp_dir = setup_test_dir();

    lnix_cmd()
        .current_dir(temp_dir.path())
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created"));
}

#[test]
fn test_init_creates_valid_yaml_structure() {
    let temp_dir = setup_test_dir();

    lnix_cmd()
        .current_dir(temp_dir.path())
        .arg("init")
        .assert()
        .success();

    let yaml_content =
        fs::read_to_string(temp_dir.path().join("lazynix.yaml")).expect("Failed to read yaml");

    assert!(is_valid_yaml().eval(&yaml_content));
    assert!(yaml_content.contains("devShell"));
    assert!(yaml_content.contains("package"));
}

#[test]
fn test_init_creates_valid_flake_structure() {
    let temp_dir = setup_test_dir();

    lnix_cmd()
        .current_dir(temp_dir.path())
        .arg("init")
        .assert()
        .success();

    let flake_content =
        fs::read_to_string(temp_dir.path().join("flake.nix")).expect("Failed to read flake");

    assert!(flake_content.contains("inputs"));
    assert!(flake_content.contains("outputs"));
}

#[test]
fn test_init_help_message() {
    lnix_cmd()
        .arg("init")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialize"))
        .stdout(predicate::str::contains("--force"));
}
