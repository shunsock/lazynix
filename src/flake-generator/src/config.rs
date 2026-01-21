use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::{FlakeGeneratorError as Error, Result};
use crate::package_validator::is_valid_package_name;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub dev_shell: DevShell,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DevShell {
    #[serde(default)]
    pub allow_unfree: bool,

    pub package: Package,

    #[serde(default)]
    pub shell_hook: Vec<String>,

    #[serde(default)]
    pub env: Option<Env>,

    #[serde(default)]
    pub test: Vec<String>,

    #[serde(default)]
    pub task: Option<HashMap<String, TaskDef>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Package {
    #[serde(default)]
    pub stable: Vec<String>,

    #[serde(default)]
    pub unstable: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvVar {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Env {
    #[serde(default)]
    pub dotenv: Vec<String>,

    #[serde(default)]
    pub envvar: Vec<EnvVar>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskDef {
    #[serde(default)]
    pub description: Option<String>,

    pub commands: Vec<String>,
}

fn is_valid_task_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    name.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

pub fn validate_config(config: &Config) -> Result<()> {
    // Validate stable package names
    for pkg in &config.dev_shell.package.stable {
        if !is_valid_package_name(pkg) {
            return Err(Error::Validation(format!(
                "Invalid stable package name: {}. Package names should contain only alphanumeric characters, hyphens, underscores, and dots (for nested attributes like python312Packages.pip)",
                pkg
            )));
        }
    }

    // Validate unstable package names
    for pkg in &config.dev_shell.package.unstable {
        if !is_valid_package_name(pkg) {
            return Err(Error::Validation(format!(
                "Invalid unstable package name: {}. Package names should contain only alphanumeric characters, hyphens, underscores, and dots (for nested attributes like python312Packages.pip)",
                pkg
            )));
        }
    }

    // Validate task definitions
    if let Some(tasks) = &config.dev_shell.task {
        for (task_name, task_def) in tasks {
            // Validate task name
            if !is_valid_task_name(task_name) {
                return Err(Error::Validation(format!(
                    "Invalid task name: '{}'. Task names should contain only alphanumeric characters, hyphens, and underscores",
                    task_name
                )));
            }

            // Validate commands is not empty
            if task_def.commands.is_empty() {
                return Err(Error::Validation(format!(
                    "Task '{}' has an empty commands list. At least one command must be specified",
                    task_name
                )));
            }
        }
    }

    // Check that at least some configuration is provided
    if config.dev_shell.package.stable.is_empty() && config.dev_shell.package.unstable.is_empty() {
        eprintln!("Warning: No packages specified in lazynix.yaml");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_yaml_with_env() {
        let yaml = r#"
devShell:
  allowUnfree: false
  package:
    stable:
      - bash
  env:
    dotenv:
      - .env
    envvar:
      - name: MY_VAR
        value: hello
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert!(config.dev_shell.env.is_some());

        let env = config.dev_shell.env.as_ref().unwrap();
        assert_eq!(env.dotenv.len(), 1);
        assert_eq!(env.dotenv[0], ".env");
        assert_eq!(env.envvar.len(), 1);
        assert_eq!(env.envvar[0].name, "MY_VAR");
        assert_eq!(env.envvar[0].value, "hello");
    }

    #[test]
    fn test_deserialize_yaml_without_env() {
        let yaml = r#"
devShell:
  allowUnfree: false
  package:
    stable:
      - bash
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert!(config.dev_shell.env.is_none());
    }

    #[test]
    fn test_deserialize_yaml_with_dotenv_only() {
        let yaml = r#"
devShell:
  allowUnfree: false
  package:
    stable:
      - bash
  env:
    dotenv:
      - .env
      - .env.local
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert!(config.dev_shell.env.is_some());

        let env = config.dev_shell.env.as_ref().unwrap();
        assert_eq!(env.dotenv.len(), 2);
        assert_eq!(env.dotenv[0], ".env");
        assert_eq!(env.dotenv[1], ".env.local");
        assert_eq!(env.envvar.len(), 0);
    }

    #[test]
    fn test_deserialize_yaml_with_envvar_only() {
        let yaml = r#"
devShell:
  allowUnfree: false
  package:
    stable:
      - bash
  env:
    envvar:
      - name: PYTHONPATH
        value: /path/to/project
      - name: MY_VAR
        value: test123
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert!(config.dev_shell.env.is_some());

        let env = config.dev_shell.env.as_ref().unwrap();
        assert_eq!(env.dotenv.len(), 0);
        assert_eq!(env.envvar.len(), 2);
        assert_eq!(env.envvar[0].name, "PYTHONPATH");
        assert_eq!(env.envvar[0].value, "/path/to/project");
        assert_eq!(env.envvar[1].name, "MY_VAR");
        assert_eq!(env.envvar[1].value, "test123");
    }

    #[test]
    fn test_deserialize_yaml_with_empty_env() {
        let yaml = r#"
devShell:
  allowUnfree: false
  package:
    stable:
      - bash
  env:
    dotenv: []
    envvar: []
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert!(config.dev_shell.env.is_some());

        let env = config.dev_shell.env.as_ref().unwrap();
        assert_eq!(env.dotenv.len(), 0);
        assert_eq!(env.envvar.len(), 0);
    }

    #[test]
    fn test_deserialize_yaml_with_test() {
        let yaml = r#"
devShell:
  allowUnfree: false
  package:
    stable:
      - bash
  test:
    - pytest
    - cargo test
    - npm run test
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.dev_shell.test.len(), 3);
        assert_eq!(config.dev_shell.test[0], "pytest");
        assert_eq!(config.dev_shell.test[1], "cargo test");
        assert_eq!(config.dev_shell.test[2], "npm run test");
    }

    #[test]
    fn test_deserialize_yaml_without_test() {
        let yaml = r#"
devShell:
  allowUnfree: false
  package:
    stable:
      - bash
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.dev_shell.test.len(), 0);
        assert!(config.dev_shell.test.is_empty());
    }

    #[test]
    fn test_deserialize_yaml_with_empty_test() {
        let yaml = r#"
devShell:
  allowUnfree: false
  package:
    stable:
      - bash
  test: []
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.dev_shell.test.len(), 0);
        assert!(config.dev_shell.test.is_empty());
    }

    #[test]
    fn test_deserialize_yaml_with_task() {
        let yaml = r#"
devShell:
  allowUnfree: false
  package:
    stable:
      - bash
  task:
    sync:
      description: "Sync dependencies"
      commands:
        - uv sync
    start:
      description: "Start application"
      commands:
        - uv run python main.py
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert!(config.dev_shell.task.is_some());

        let tasks = config.dev_shell.task.as_ref().unwrap();
        assert_eq!(tasks.len(), 2);

        let sync_task = tasks.get("sync").unwrap();
        assert_eq!(sync_task.description, Some("Sync dependencies".to_string()));
        assert_eq!(sync_task.commands.len(), 1);
        assert_eq!(sync_task.commands[0], "uv sync");

        let start_task = tasks.get("start").unwrap();
        assert_eq!(
            start_task.description,
            Some("Start application".to_string())
        );
        assert_eq!(start_task.commands.len(), 1);
        assert_eq!(start_task.commands[0], "uv run python main.py");
    }

    #[test]
    fn test_deserialize_yaml_without_task() {
        let yaml = r#"
devShell:
  allowUnfree: false
  package:
    stable:
      - bash
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert!(config.dev_shell.task.is_none());
    }

    #[test]
    fn test_deserialize_yaml_with_task_no_description() {
        let yaml = r#"
devShell:
  allowUnfree: false
  package:
    stable:
      - bash
  task:
    hello:
      commands:
        - echo "Hello, LazyNix!"
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert!(config.dev_shell.task.is_some());

        let tasks = config.dev_shell.task.as_ref().unwrap();
        let hello_task = tasks.get("hello").unwrap();
        assert_eq!(hello_task.description, None);
        assert_eq!(hello_task.commands.len(), 1);
        assert_eq!(hello_task.commands[0], "echo \"Hello, LazyNix!\"");
    }

    #[test]
    fn test_validate_task_valid() {
        let yaml = r#"
devShell:
  allowUnfree: false
  package:
    stable:
      - bash
  task:
    my-task_123:
      description: "Valid task"
      commands:
        - echo "test"
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_validate_task_invalid_name() {
        let yaml = r#"
devShell:
  allowUnfree: false
  package:
    stable:
      - bash
  task:
    invalid@task:
      commands:
        - echo "test"
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid task name"));
    }

    #[test]
    fn test_validate_task_empty_command() {
        let yaml = r#"
devShell:
  allowUnfree: false
  package:
    stable:
      - bash
  task:
    empty-task:
      description: "Empty command"
      commands: []
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("empty commands list"));
    }
}
