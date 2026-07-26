//! `lnix init` — scaffold a new project from bundled templates.

use crate::deps::Deps;
use crate::error::ApplicationError;
use crate::event::UseCaseEvent;

/// Writes the starter `lazynix.yaml` and `flake.nix`.
///
/// Without `force`, refuses to overwrite either existing file.
pub fn init(d: &Deps, force: bool) -> Result<i32, ApplicationError> {
    if !force {
        if d.scaffolder.config_exists() {
            return Err(ApplicationError::FileExists(
                d.scaffolder.config_path_display(),
            ));
        }
        if d.scaffolder.flake_exists() {
            return Err(ApplicationError::FileExists(
                d.scaffolder.flake_path_display(),
            ));
        }
    }

    d.scaffolder.write_config_template()?;
    d.scaffolder.write_flake_template()?;

    d.reporter.report(&UseCaseEvent::ProjectInitialized {
        config_path: d.scaffolder.config_path_display(),
        flake_path: d.scaffolder.flake_path_display(),
    });
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mocks::*;

    #[test]
    fn scaffolds_both_files_and_prints_next_steps() {
        let m = Mocks::with_missing_config();

        let code = init(&m.deps(), false).unwrap();

        assert_eq!(code, 0);
        assert!(m.scaffolder.config_written());
        assert!(m.scaffolder.flake_written());
        assert!(m.reporter.events().iter().any(|e| matches!(
            e,
            UseCaseEvent::ProjectInitialized { config_path, flake_path }
            if config_path == "./lazynix.yaml" && flake_path == "./flake.nix"
        )));
    }

    #[test]
    fn refuses_existing_config_without_force() {
        let m = Mocks::with_missing_config().with_existing_scaffold(true, false);

        let result = init(&m.deps(), false);

        assert!(matches!(result, Err(ApplicationError::FileExists(_))));
        assert!(!m.scaffolder.config_written());
    }

    #[test]
    fn force_overwrites_existing_files() {
        let m = Mocks::with_missing_config().with_existing_scaffold(true, true);

        let code = init(&m.deps(), true).unwrap();

        assert_eq!(code, 0);
        assert!(m.scaffolder.config_written());
        assert!(m.scaffolder.flake_written());
    }
}
