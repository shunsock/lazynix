use crate::definition::dev_shell::DevShellDefinition;
use crate::error::{Diagnostic, ValidationError};

/// Checks cross-field constraints that value objects cannot express.
///
/// Field-level invariants (package names, task names, env var names,
/// versions) are already guaranteed by the type system at this point.
///
/// Returns non-fatal findings as [`Diagnostic`] values instead of
/// printing them, so this function stays free of I/O; the caller
/// decides how to display them.
pub fn validate_config(config: &DevShellDefinition) -> Result<Vec<Diagnostic>, ValidationError> {
    if let Some(tasks) = &config.dev_shell.task {
        for (task_name, task_def) in tasks {
            if task_def.commands.is_empty() {
                return Err(ValidationError::EmptyTaskCommands(
                    task_name.as_str().to_string(),
                ));
            }
        }
    }

    let mut diagnostics = Vec::new();
    let package = &config.dev_shell.package;
    let has_no_packages =
        package.stable.is_empty() && package.unstable.is_empty() && package.pinned.is_empty();
    if has_no_packages {
        diagnostics.push(Diagnostic::NoPackages);
    }

    Ok(diagnostics)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_config_with_valid_task() {
        // Arrange
        let yaml = r#"
devShell:
  package:
    stable:
      - name: bash
  task:
    sync:
      commands:
        - uv sync
"#;
        let config: DevShellDefinition = serde_yaml::from_str(yaml).unwrap();

        // Act
        let result = validate_config(&config);

        // Assert
        assert_eq!(result.unwrap(), Vec::<Diagnostic>::new());
    }

    #[test]
    fn reports_no_packages_as_diagnostic() {
        // Arrange
        let yaml = r#"
devShell:
  package:
    stable: []
"#;
        let config: DevShellDefinition = serde_yaml::from_str(yaml).unwrap();

        // Act
        let result = validate_config(&config);

        // Assert
        assert_eq!(result.unwrap(), vec![Diagnostic::NoPackages]);
    }

    #[test]
    fn rejects_task_with_empty_commands() {
        // Arrange
        let yaml = r#"
devShell:
  package:
    stable:
      - name: bash
  task:
    empty-task:
      commands: []
"#;
        let config: DevShellDefinition = serde_yaml::from_str(yaml).unwrap();

        // Act
        let result = validate_config(&config);

        // Assert
        assert_eq!(
            result.unwrap_err(),
            ValidationError::EmptyTaskCommands("empty-task".to_string())
        );
    }
}
