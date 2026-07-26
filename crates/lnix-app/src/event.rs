//! Semantic events emitted by use-cases during execution.
//!
//! Use-cases speak in terms of *what happened* (a config was read,
//! a lock was updated, a task started). Presentation adapters map
//! these events to human-readable output. Keeping the vocabulary here
//! means i18n, JSON output, or a TUI can be added by swapping the
//! adapter — no use-case touches string literals.
//!
//! Variant naming convention: gerund ("Reading", "Validating",
//! "Entering", "Using") marks the start of an action; past participle
//! ("Updated", "Generated", "Initialized") marks its completion; other
//! forms describe a transient state ("NoPackagesToValidate").

use lnix_domain::{PackageName, PackageVersion, TaskName, ValidationResult};

#[derive(Debug, Clone, PartialEq)]
pub enum UseCaseEvent {
    ReadingConfig,
    ValidatingConfig,
    ConfigDiagnostic(String),
    ResolvingPinned {
        name: PackageName,
        version: PackageVersion,
    },
    UpdatedYamlWithResolvedVersions,
    GeneratingFlake,
    FlakeGenerated,
    UpdatingLock,
    LockUpdated,
    LockUpdateSkipped,
    EnteringDevelopShell,
    RunningCommand {
        argv: Vec<String>,
    },
    UsingExistingFlake,
    RunningTask {
        name: TaskName,
        description: Option<String>,
    },
    NoPackagesToValidate,
    ValidationReport {
        result: ValidationResult,
        verbose: bool,
    },
    ProjectInitialized {
        config_path: String,
        flake_path: String,
    },
    SearchResults(String),
    EnteringTestRun,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_reading_config_events_compare_equal() {
        let left = UseCaseEvent::ReadingConfig;
        let right = UseCaseEvent::ReadingConfig;

        assert_eq!(left, right);
    }

    #[test]
    fn running_command_events_with_different_argv_compare_unequal() {
        let left = UseCaseEvent::RunningCommand {
            argv: vec!["a".to_string()],
        };
        let right = UseCaseEvent::RunningCommand {
            argv: vec!["b".to_string()],
        };

        assert_ne!(left, right);
    }

    #[test]
    fn identical_entering_test_run_events_compare_equal() {
        let left = UseCaseEvent::EnteringTestRun;
        let right = UseCaseEvent::EnteringTestRun;

        assert_eq!(left, right);
    }
}
