//! `lnix init` — scaffold a new project from bundled templates.

use crate::deps::Deps;
use crate::error::ApplicationError;

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

    let yaml_path = d.scaffolder.config_path_display();
    d.out.info("✓ Initialized LazyNix project");
    d.out.info(&format!("  - Created: {}", yaml_path));
    d.out.info(&format!(
        "  - Created: {}",
        d.scaffolder.flake_path_display()
    ));
    d.out.info("");
    d.out.info("Next steps:");
    d.out.info(&format!(
        "  1. Run 'git add flake.nix {}' to stage the files",
        yaml_path
    ));
    d.out.info(&format!(
        "  2. Edit {} to configure your environment",
        yaml_path
    ));
    d.out
        .info("  3. Run 'lnix develop' to generate flake.nix and enter the shell");
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mocks::*;

    #[test]
    fn scaffolds_both_files_and_prints_next_steps() {
        // Arrange
        let m = Mocks::with_missing_config();

        // Act
        let code = init(&m.deps(), false).unwrap();

        // Assert
        assert_eq!(code, 0);
        assert!(m.scaffolder.config_written());
        assert!(m.scaffolder.flake_written());
        assert!(
            m.out
                .infos()
                .iter()
                .any(|line| line.contains("Initialized LazyNix project"))
        );
        assert!(m.out.infos().iter().any(|line| line.contains("Created")));
    }

    #[test]
    fn refuses_existing_config_without_force() {
        // Arrange
        let m = Mocks::with_missing_config().with_existing_scaffold(true, false);

        // Act
        let result = init(&m.deps(), false);

        // Assert
        assert!(matches!(result, Err(ApplicationError::FileExists(_))));
        assert!(!m.scaffolder.config_written());
    }

    #[test]
    fn force_overwrites_existing_files() {
        // Arrange
        let m = Mocks::with_missing_config().with_existing_scaffold(true, true);

        // Act
        let code = init(&m.deps(), true).unwrap();

        // Assert
        assert_eq!(code, 0);
        assert!(m.scaffolder.config_written());
        assert!(m.scaffolder.flake_written());
    }
}
