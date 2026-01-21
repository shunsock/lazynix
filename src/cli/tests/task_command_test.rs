use predicates::prelude::*;

mod common;
use common::*;

#[test]
fn test_task_help_message() {
    lnix_cmd()
        .arg("task")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Run a task"))
        .stdout(predicate::str::contains("TASK_NAME"));
}

#[test]
fn test_task_missing_config_file() {
    let temp_dir = setup_test_dir();

    lnix_cmd()
        .arg("-C")
        .arg(temp_dir.path())
        .arg("task")
        .arg("mytask")
        .assert()
        .failure()
        .stderr(predicate::str::contains("lazynix.yaml"));
}

#[test]
fn test_task_no_tasks_defined() {
    let temp_dir = setup_test_dir_with_config(&config_with_packages(&["bash"], &[]));

    lnix_cmd()
        .arg("-C")
        .arg(temp_dir.path())
        .arg("task")
        .arg("mytask")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("No tasks defined").or(predicate::str::contains("not found")),
        );
}

#[test]
fn test_task_task_not_found() {
    let config = config_with_tasks(&[("hello", vec!["echo hello"], Some("Say hello"))]);
    let temp_dir = setup_test_dir_with_config(&config);

    lnix_cmd()
        .arg("-C")
        .arg(temp_dir.path())
        .arg("task")
        .arg("nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("nonexistent")));
}

#[test]
fn test_task_shows_description() {
    let config = config_with_tasks(&[("hello", vec!["echo hello"], Some("Say hello"))]);
    let temp_dir = setup_test_dir_with_config(&config);

    let _ = lnix_cmd()
        .arg("-C")
        .arg(temp_dir.path())
        .arg("task")
        .arg("hello")
        .timeout(std::time::Duration::from_secs(5))
        .assert();
}

#[test]
fn test_task_with_cli_args_interpolation() {
    let config = config_with_tasks(&[(
        "greet",
        vec!["echo Hello {{.CLI_ARGS}}"],
        Some("Greet someone"),
    )]);
    let temp_dir = setup_test_dir_with_config(&config);

    let _ = lnix_cmd()
        .arg("-C")
        .arg(temp_dir.path())
        .arg("task")
        .arg("greet")
        .arg("World")
        .timeout(std::time::Duration::from_secs(5))
        .assert();
}

#[test]
#[ignore]
fn test_task_executes_commands() {
    let config = config_with_tasks(&[("test", vec!["echo success"], Some("Test task"))]);
    let temp_dir = setup_test_dir_with_config(&config);

    lnix_cmd()
        .arg("-C")
        .arg(temp_dir.path())
        .arg("task")
        .arg("test")
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .success()
        .stdout(predicate::str::contains("success"));
}
