//! `lnix task` — run a named task defined in `lazynix.yaml`.

use lnix_domain::{TaskName, interpolate_command, validate_config};

use crate::deps::Deps;
use crate::error::ApplicationError;
use crate::event::UseCaseEvent;

/// Looks up `task_name`, interpolates `args` into its commands, and
/// runs them sequentially in `nix develop`. Returns the task's exit
/// code.
///
/// Unlike the flake-generating commands, tasks read the config as-is:
/// no pinned resolution, no dotenv check, no flake regeneration.
pub fn task(d: &Deps, task_name: &str, args: &[String]) -> Result<i32, ApplicationError> {
    let task_name: TaskName = task_name.parse()?;

    d.reporter.report(&UseCaseEvent::ReadingConfig);
    let config = d.repo.read_config()?;

    d.reporter.report(&UseCaseEvent::ValidatingConfig);
    for diagnostic in validate_config(&config).map_err(lnix_domain::ConfigError::from)? {
        d.reporter
            .report(&UseCaseEvent::ConfigDiagnostic(diagnostic.to_string()));
    }

    let tasks = config
        .dev_shell
        .task
        .ok_or(ApplicationError::NoTasksDefined)?;
    let task_def = tasks
        .get(&task_name)
        .ok_or_else(|| ApplicationError::TaskNotFound(task_name.to_string()))?;

    let commands = interpolate_command(&task_def.commands, args);

    d.reporter.report(&UseCaseEvent::RunningTask {
        name: task_name.clone(),
        description: task_def.description.clone(),
    });

    Ok(d.nix.run_task(&commands)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mocks::*;

    const TASK_CONFIG: &str = "devShell:\n  package:\n    stable:\n      - name: bash\n  task:\n    greet:\n      description: Say hi\n      commands:\n        - echo hi {{.CLI_ARGS}}\n";

    #[test]
    fn interpolates_args_and_runs_the_task() {
        let m = Mocks::with_config(config_from_yaml(TASK_CONFIG));

        let code = task(&m.deps(), "greet", &["world".to_string()]).unwrap();

        assert_eq!(code, 0);
        assert_eq!(
            m.nix.run_task_commands().unwrap(),
            vec!["echo hi world".to_string()]
        );
        assert!(m.reporter.events().iter().any(|e| matches!(
            e,
            UseCaseEvent::RunningTask { name, .. } if name.as_str() == "greet"
        )));
    }

    #[test]
    fn errors_when_no_tasks_are_defined() {
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable:\n      - name: bash\n",
        ));

        let result = task(&m.deps(), "greet", &[]);

        assert!(matches!(result, Err(ApplicationError::NoTasksDefined)));
    }

    #[test]
    fn errors_when_the_task_is_not_found() {
        let m = Mocks::with_config(config_from_yaml(TASK_CONFIG));

        let result = task(&m.deps(), "nonexistent", &[]);

        assert!(
            matches!(result, Err(ApplicationError::TaskNotFound(name)) if name == "nonexistent")
        );
    }

    #[test]
    fn rejects_invalid_task_name_before_reading_config() {
        let m = Mocks::with_missing_config();

        let result = task(&m.deps(), "bad@name", &[]);

        assert!(matches!(result, Err(ApplicationError::InvalidInput(_))));
    }
}
