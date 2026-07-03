use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::definition::env::Env;
use crate::definition::package::Package;
use crate::definition::task::TaskDef;
use crate::values::TaskName;

/// Root of the `lazynix.yaml` document.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DevShellDefinition {
    pub dev_shell: DevShell,
}

/// The `devShell` section: everything needed to render a dev shell flake.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DevShell {
    #[serde(default = "default_allow_unfree")]
    pub allow_unfree: bool,

    pub package: Package,

    #[serde(default)]
    pub shell_hook: Vec<String>,

    #[serde(default)]
    pub env: Option<Env>,

    #[serde(default)]
    pub test: Vec<String>,

    #[serde(default)]
    pub task: Option<HashMap<TaskName, TaskDef>>,

    /// Shell alias files to load.
    /// Alias definitions are extracted from the specified files.
    #[serde(default)]
    pub shell_alias: Vec<String>,
}

fn default_allow_unfree() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_minimal_yaml() {
        // Arrange
        let yaml = r#"
devShell:
  package:
    stable:
      - name: bash
"#;

        // Act
        let config: DevShellDefinition = serde_yaml::from_str(yaml).unwrap();

        // Assert
        assert!(
            config.dev_shell.allow_unfree,
            "allowUnfree defaults to true"
        );
        assert_eq!(config.dev_shell.package.stable.len(), 1);
        assert!(config.dev_shell.env.is_none());
        assert!(config.dev_shell.task.is_none());
        assert!(config.dev_shell.test.is_empty());
        assert!(config.dev_shell.shell_alias.is_empty());
    }

    #[test]
    fn deserializes_task_section_with_validated_names() {
        // Arrange
        let yaml = r#"
devShell:
  package:
    stable:
      - name: bash
  task:
    my-task_123:
      description: "Valid task"
      commands:
        - echo "test"
"#;

        // Act
        let config: DevShellDefinition = serde_yaml::from_str(yaml).unwrap();

        // Assert
        let tasks = config.dev_shell.task.unwrap();
        let task = tasks.get(&"my-task_123".parse().unwrap()).unwrap();
        assert_eq!(task.description.as_deref(), Some("Valid task"));
        assert_eq!(task.commands, vec!["echo \"test\""]);
    }

    #[test]
    fn rejects_invalid_task_name_at_parse_time() {
        // Arrange
        let yaml = r#"
devShell:
  package:
    stable:
      - name: bash
  task:
    invalid@task:
      commands:
        - echo "test"
"#;

        // Act
        let result = serde_yaml::from_str::<DevShellDefinition>(yaml);

        // Assert
        let message = result.unwrap_err().to_string();
        assert!(message.contains("Invalid task name"), "got: {message}");
    }
}
