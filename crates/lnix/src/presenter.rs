//! Terminal presentation adapter for [`UseCaseEvent`]s.
//!
//! The composition root wires this adapter as the concrete
//! [`ReporterPort`] implementation. Rendering logic lives in the pure
//! function [`render_event`] so it can be exercised without stdout
//! capture; [`TerminalPresenter::report`] is a thin write-through that
//! sends stdout lines to `println!` and the single stderr variant
//! ([`UseCaseEvent::ConfigDiagnostic`]) to `eprintln!`.
//!
//! Placement note: this adapter lives in the `lnix` bin crate rather
//! than `lnix-infra` because `ReporterPort` is defined in `lnix-app`
//! and `lnix-infra` does not depend on `lnix-app`. Other adapters
//! (persistence, gateway) target domain ports and live in `lnix-infra`.

use lnix_app::{ReporterPort, UseCaseEvent};
use lnix_domain::{format_validation_result, format_validation_result_verbose};

pub struct TerminalPresenter;

impl ReporterPort for TerminalPresenter {
    fn report(&self, event: &UseCaseEvent) {
        if let UseCaseEvent::ConfigDiagnostic(message) = event {
            eprintln!("Warning: {message}");
            return;
        }
        for line in render_event(event) {
            println!("{line}");
        }
    }
}

fn render_event(event: &UseCaseEvent) -> Vec<String> {
    match event {
        UseCaseEvent::ReadingConfig => vec!["Reading configuration...".to_string()],
        UseCaseEvent::ValidatingConfig => vec!["Validating configuration...".to_string()],
        // SEE: report() で ConfigDiagnostic は早期リターン、この arm は exhaustiveness 用
        UseCaseEvent::ConfigDiagnostic(_) => Vec::new(),
        UseCaseEvent::ResolvingPinned { name, version } => {
            vec![format!("Resolving version for {name} @ {version}...")]
        }
        UseCaseEvent::UpdatedYamlWithResolvedVersions => {
            vec!["Updated lazynix.yaml with resolved versions".to_string()]
        }
        UseCaseEvent::GeneratingFlake => vec!["Generating flake.nix...".to_string()],
        UseCaseEvent::FlakeGenerated => vec!["✓ flake.nix generated successfully".to_string()],
        UseCaseEvent::UpdatingLock => vec!["Updating flake.lock...".to_string()],
        UseCaseEvent::LockUpdated => vec!["flake.lock updated successfully".to_string()],
        UseCaseEvent::LockUpdateSkipped => {
            vec!["Skipping flake.lock update (use --update to update)".to_string()]
        }
        UseCaseEvent::EnteringDevelopShell => {
            vec![String::new(), "Entering nix develop shell...".to_string()]
        }
        UseCaseEvent::RunningCommand { argv } => vec![
            String::new(),
            format!("Running command: {}", argv.join(" ")),
        ],
        UseCaseEvent::UsingExistingFlake => {
            vec!["Using existing flake.nix (--no-regen specified)".to_string()]
        }
        UseCaseEvent::RunningTask { name, description } => {
            let mut lines = vec![format!("Running task: {name}")];
            if let Some(desc) = description
                && !desc.is_empty()
            {
                lines.push(format!("Description: {desc}"));
            }
            lines.push(String::new());
            lines
        }
        UseCaseEvent::NoPackagesToValidate => vec!["No packages to validate.".to_string()],
        UseCaseEvent::ValidationReport { result, verbose } => {
            let text = if *verbose {
                format_validation_result_verbose(result)
            } else {
                format_validation_result(result)
            };
            vec![text.trim_end().to_string()]
        }
        UseCaseEvent::ProjectInitialized {
            config_path,
            flake_path,
        } => vec![
            "✓ Initialized LazyNix project".to_string(),
            format!("  - Created: {config_path}"),
            format!("  - Created: {flake_path}"),
            String::new(),
            "Next steps:".to_string(),
            format!("  1. Run 'git add flake.nix {config_path}' to stage the files"),
            format!("  2. Edit {config_path} to configure your environment"),
            "  3. Run 'lnix develop' to generate flake.nix and enter the shell".to_string(),
        ],
        UseCaseEvent::SearchResults(output) => vec![output.trim_end().to_string()],
        UseCaseEvent::EnteringTestRun => {
            vec![String::new(), "Running tests...".to_string()]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lnix_domain::{PackageValidationError, ValidationResult};

    #[test]
    fn renders_reading_config_as_single_line() {
        assert_eq!(
            render_event(&UseCaseEvent::ReadingConfig),
            vec!["Reading configuration...".to_string()]
        );
    }

    #[test]
    fn renders_validating_config_as_single_line() {
        assert_eq!(
            render_event(&UseCaseEvent::ValidatingConfig),
            vec!["Validating configuration...".to_string()]
        );
    }

    #[test]
    fn renders_config_diagnostic_as_empty_because_it_targets_stderr() {
        assert_eq!(
            render_event(&UseCaseEvent::ConfigDiagnostic("boom".to_string())),
            Vec::<String>::new()
        );
    }

    #[test]
    fn renders_resolving_pinned_with_name_and_version() {
        let event = UseCaseEvent::ResolvingPinned {
            name: "hello".parse().unwrap(),
            version: "2.12".parse().unwrap(),
        };
        assert_eq!(
            render_event(&event),
            vec!["Resolving version for hello @ 2.12...".to_string()]
        );
    }

    #[test]
    fn renders_updated_yaml_with_resolved_versions_line() {
        assert_eq!(
            render_event(&UseCaseEvent::UpdatedYamlWithResolvedVersions),
            vec!["Updated lazynix.yaml with resolved versions".to_string()]
        );
    }

    #[test]
    fn renders_generating_flake_line() {
        assert_eq!(
            render_event(&UseCaseEvent::GeneratingFlake),
            vec!["Generating flake.nix...".to_string()]
        );
    }

    #[test]
    fn renders_flake_generated_success_line() {
        assert_eq!(
            render_event(&UseCaseEvent::FlakeGenerated),
            vec!["✓ flake.nix generated successfully".to_string()]
        );
    }

    #[test]
    fn renders_updating_lock_line() {
        assert_eq!(
            render_event(&UseCaseEvent::UpdatingLock),
            vec!["Updating flake.lock...".to_string()]
        );
    }

    #[test]
    fn renders_lock_updated_line() {
        assert_eq!(
            render_event(&UseCaseEvent::LockUpdated),
            vec!["flake.lock updated successfully".to_string()]
        );
    }

    #[test]
    fn renders_lock_update_skipped_line() {
        assert_eq!(
            render_event(&UseCaseEvent::LockUpdateSkipped),
            vec!["Skipping flake.lock update (use --update to update)".to_string()]
        );
    }

    #[test]
    fn renders_entering_develop_shell_with_leading_blank_line() {
        assert_eq!(
            render_event(&UseCaseEvent::EnteringDevelopShell),
            vec![String::new(), "Entering nix develop shell...".to_string(),]
        );
    }

    #[test]
    fn renders_running_command_with_argv_joined_by_space() {
        let event = UseCaseEvent::RunningCommand {
            argv: vec!["echo".to_string(), "hi".to_string()],
        };
        assert_eq!(
            render_event(&event),
            vec![String::new(), "Running command: echo hi".to_string(),]
        );
    }

    #[test]
    fn renders_using_existing_flake_line() {
        assert_eq!(
            render_event(&UseCaseEvent::UsingExistingFlake),
            vec!["Using existing flake.nix (--no-regen specified)".to_string()]
        );
    }

    #[test]
    fn renders_running_task_with_description_when_present() {
        let event = UseCaseEvent::RunningTask {
            name: "build".parse().unwrap(),
            description: Some("Compile everything".to_string()),
        };
        assert_eq!(
            render_event(&event),
            vec![
                "Running task: build".to_string(),
                "Description: Compile everything".to_string(),
                String::new(),
            ]
        );
    }

    #[test]
    fn renders_running_task_without_description_line_when_absent() {
        let event = UseCaseEvent::RunningTask {
            name: "build".parse().unwrap(),
            description: None,
        };
        assert_eq!(
            render_event(&event),
            vec!["Running task: build".to_string(), String::new()]
        );
    }

    #[test]
    fn renders_running_task_omits_empty_description() {
        let event = UseCaseEvent::RunningTask {
            name: "build".parse().unwrap(),
            description: Some(String::new()),
        };
        assert_eq!(
            render_event(&event),
            vec!["Running task: build".to_string(), String::new()]
        );
    }

    #[test]
    fn renders_no_packages_to_validate_line() {
        assert_eq!(
            render_event(&UseCaseEvent::NoPackagesToValidate),
            vec!["No packages to validate.".to_string()]
        );
    }

    #[test]
    fn renders_validation_report_success_using_domain_formatter() {
        let result = ValidationResult {
            valid_packages: vec!["vim".to_string()],
            errors: vec![],
        };
        let event = UseCaseEvent::ValidationReport {
            result: result.clone(),
            verbose: false,
        };
        let rendered = render_event(&event);
        assert_eq!(rendered.len(), 1);
        assert!(rendered[0].contains("✓"));
        assert!(rendered[0].contains("1 package(s)"));
        assert!(!rendered[0].ends_with('\n'));
    }

    #[test]
    fn renders_validation_report_verbose_appends_debug_details() {
        let result = ValidationResult {
            valid_packages: vec![],
            errors: vec![PackageValidationError::PackageNotFound {
                package: "nonexistent".to_string(),
            }],
        };
        let event = UseCaseEvent::ValidationReport {
            result,
            verbose: true,
        };
        let rendered = render_event(&event);
        assert_eq!(rendered.len(), 1);
        assert!(rendered[0].contains("Verbose Error Details"));
        assert!(rendered[0].contains("PackageNotFound"));
    }

    #[test]
    fn renders_project_initialized_with_next_steps_block() {
        let event = UseCaseEvent::ProjectInitialized {
            config_path: "lazynix.yaml".to_string(),
            flake_path: "flake.nix".to_string(),
        };
        assert_eq!(
            render_event(&event),
            vec![
                "✓ Initialized LazyNix project".to_string(),
                "  - Created: lazynix.yaml".to_string(),
                "  - Created: flake.nix".to_string(),
                String::new(),
                "Next steps:".to_string(),
                "  1. Run 'git add flake.nix lazynix.yaml' to stage the files".to_string(),
                "  2. Edit lazynix.yaml to configure your environment".to_string(),
                "  3. Run 'lnix develop' to generate flake.nix and enter the shell".to_string(),
            ]
        );
    }

    #[test]
    fn renders_search_results_trimming_trailing_newlines() {
        let event = UseCaseEvent::SearchResults("hello 2.12\nvim 9.0\n".to_string());
        assert_eq!(
            render_event(&event),
            vec!["hello 2.12\nvim 9.0".to_string()]
        );
    }

    #[test]
    fn renders_entering_test_run_with_leading_blank_line() {
        assert_eq!(
            render_event(&UseCaseEvent::EnteringTestRun),
            vec![String::new(), "Running tests...".to_string()]
        );
    }
}
