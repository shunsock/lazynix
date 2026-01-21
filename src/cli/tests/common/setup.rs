use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

/// Create a Command for the lnix binary
#[allow(deprecated)]
pub fn lnix_cmd() -> Command {
    Command::cargo_bin("lnix").unwrap()
}

/// Create a temporary test directory
#[allow(dead_code)]
pub fn setup_test_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

/// Create a temporary test directory with a lazynix.yaml config file
#[allow(dead_code)]
pub fn setup_test_dir_with_config(config: &str) -> TempDir {
    let temp_dir = setup_test_dir();
    let config_path = temp_dir.path().join("lazynix.yaml");
    fs::write(config_path, config).expect("Failed to write config file");
    temp_dir
}
