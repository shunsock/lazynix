use predicates::prelude::*;

mod common;
use common::*;

#[test]
fn test_help_message() {
    lnix_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("LazyNix"))
        .stdout(predicate::str::contains("Commands:"))
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("develop"))
        .stdout(predicate::str::contains("test"))
        .stdout(predicate::str::contains("update"))
        .stdout(predicate::str::contains("run"))
        .stdout(predicate::str::contains("task"))
        .stdout(predicate::str::contains("lint"));
}

#[test]
fn test_version_flag() {
    lnix_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("lnix"));
}

#[test]
fn test_version_flag_short() {
    lnix_cmd()
        .arg("-V")
        .assert()
        .success()
        .stdout(predicate::str::contains("lnix"));
}

#[test]
fn test_subcommand_help() {
    lnix_cmd()
        .arg("init")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialize"))
        .stdout(predicate::str::contains("--force"));
}

#[test]
fn test_invalid_subcommand() {
    lnix_cmd()
        .arg("invalid-command")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized subcommand"));
}

#[test]
fn test_no_subcommand_shows_help() {
    lnix_cmd()
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage:"));
}

#[test]
fn test_help_shows_all_commands() {
    lnix_cmd().arg("--help").assert().success().stdout(
        predicate::str::contains("init")
            .and(predicate::str::contains("develop"))
            .and(predicate::str::contains("test"))
            .and(predicate::str::contains("update"))
            .and(predicate::str::contains("run"))
            .and(predicate::str::contains("task"))
            .and(predicate::str::contains("lint")),
    );
}
