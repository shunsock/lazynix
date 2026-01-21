use predicates::prelude::*;

mod common;
use common::assertions::contains_package_validation_error;
use common::*;

#[test]
fn test_lint_command_all_valid_packages() {
    let temp_dir = setup_test_dir_with_config(&config_with_packages(&["hello", "vim"], &[]));

    lnix_cmd()
        .arg("-C")
        .arg(temp_dir.path())
        .arg("lint")
        .assert()
        .success()
        .stdout(predicate::str::contains("✓"))
        .stdout(predicate::str::contains("successfully"));
}

#[test]
fn test_lint_command_invalid_packages() {
    let temp_dir = setup_test_dir_with_config(&config_with_invalid_package_names());

    lnix_cmd()
        .arg("-C")
        .arg(temp_dir.path())
        .arg("lint")
        .assert()
        .failure()
        .code(1)
        .stdout(contains_package_validation_error(
            "nonexistent-pkg-xyz-12345",
        ));
}

#[test]
fn test_lint_command_mixed_packages() {
    let temp_dir =
        setup_test_dir_with_config(&config_with_packages(&["hello", "nonexistent-xyz"], &[]));

    lnix_cmd()
        .arg("-C")
        .arg(temp_dir.path())
        .arg("lint")
        .assert()
        .failure()
        .stdout(predicate::str::contains("PACKAGE_NOT_FOUND"));
}

#[test]
fn test_lint_command_verbose_flag() {
    let temp_dir = setup_test_dir_with_config(&config_with_packages(&["nonexistent-pkg"], &[]));

    lnix_cmd()
        .arg("-C")
        .arg(temp_dir.path())
        .arg("lint")
        .arg("--verbose")
        .assert()
        .failure()
        .stdout(
            predicate::str::contains("Verbose Error Details")
                .or(predicate::str::contains("PackageNotFound")),
        );
}

#[test]
fn test_lint_command_arch_flag() {
    let temp_dir = setup_test_dir_with_config(&config_with_packages(&["hello"], &[]));

    let _ = lnix_cmd()
        .arg("-C")
        .arg(temp_dir.path())
        .arg("lint")
        .arg("--arch")
        .arg("x86_64-linux")
        .assert();
}

#[test]
fn test_lint_command_empty_package_list() {
    let temp_dir = setup_test_dir_with_config(&config_with_packages(&[], &[]));

    lnix_cmd()
        .arg("-C")
        .arg(temp_dir.path())
        .arg("lint")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("No packages to validate")
                .or(predicate::str::contains("0 package")),
        );
}

#[test]
fn test_lint_command_stable_and_unstable_packages() {
    let temp_dir = setup_test_dir_with_config(&config_with_packages(&["hello"], &["vim"]));

    lnix_cmd()
        .arg("-C")
        .arg(temp_dir.path())
        .arg("lint")
        .assert()
        .success()
        .stdout(predicate::str::contains("2 package"));
}

#[test]
fn test_lint_command_help() {
    lnix_cmd()
        .arg("lint")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Validate packages"))
        .stdout(predicate::str::contains("--verbose"))
        .stdout(predicate::str::contains("--arch"));
}

#[test]
fn test_lint_command_config_not_found() {
    let temp_dir = setup_test_dir();

    lnix_cmd()
        .arg("-C")
        .arg(temp_dir.path())
        .arg("lint")
        .assert()
        .failure()
        .stderr(predicate::str::contains("lazynix.yaml"));
}

#[test]
fn test_lint_command_invalid_yaml_syntax() {
    let temp_dir = setup_test_dir_with_config(&invalid_yaml_config());

    lnix_cmd()
        .arg("-C")
        .arg(temp_dir.path())
        .arg("lint")
        .assert()
        .failure();
}
