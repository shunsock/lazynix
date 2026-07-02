//! `lnix develop` — generate the flake and enter the dev shell.

use crate::deps::Deps;
use crate::error::ApplicationError;
use crate::pipeline;

/// Renders `flake.nix`, optionally updates the lock, and enters
/// `nix develop`. Returns the process exit code.
pub fn develop(d: &Deps, update_lock: bool) -> Result<i32, ApplicationError> {
    let loaded = pipeline::load_config(d)?;
    pipeline::write_flake(d, &loaded)?;
    pipeline::maybe_update_lock(d, update_lock)?;

    d.out.info("");
    d.out.info("Entering nix develop shell...");
    d.nix.develop()?;
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mocks::*;
    use lnix_domain::ConfigError;

    #[test]
    fn writes_flake_and_enters_shell() {
        // Arrange
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable:\n      - name: bash\n",
        ));

        // Act
        let code = develop(&m.deps(), false).unwrap();

        // Assert
        assert_eq!(code, 0);
        assert!(m.writer.written().unwrap().contains("bash"));
        assert_eq!(m.nix.develop_calls(), 1);
        assert!(
            m.out
                .infos()
                .contains(&"✓ flake.nix generated successfully".to_string())
        );
        assert!(
            m.out
                .infos()
                .contains(&"Skipping flake.lock update (use --update to update)".to_string())
        );
    }

    #[test]
    fn updates_lock_when_requested() {
        // Arrange
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable:\n      - name: bash\n",
        ));

        // Act
        let code = develop(&m.deps(), true).unwrap();

        // Assert
        assert_eq!(code, 0);
        assert_eq!(m.nix.flake_update_calls(), 1);
        assert!(
            m.out
                .infos()
                .contains(&"flake.lock updated successfully".to_string())
        );
    }

    #[test]
    fn missing_config_short_circuits_before_any_side_effect() {
        // Arrange
        let m = Mocks::with_missing_config();

        // Act
        let result = develop(&m.deps(), false);

        // Assert
        assert!(matches!(
            result,
            Err(ApplicationError::Config(ConfigError::NotFound(_)))
        ));
        assert!(m.writer.written().is_none());
        assert_eq!(m.nix.develop_calls(), 0);
    }

    #[test]
    fn missing_dotenv_fails_before_writing_flake() {
        // Arrange
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable:\n      - name: bash\n  env:\n    dotenv:\n      - .env\n",
        ))
        .with_missing_env_files();

        // Act
        let result = develop(&m.deps(), false);

        // Assert
        assert!(matches!(
            result,
            Err(ApplicationError::Config(ConfigError::DotenvFileNotFound(_)))
        ));
        assert!(m.writer.written().is_none());
    }

    #[test]
    fn resolves_pinned_packages_and_persists_them() {
        // Arrange
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable:\n      - name: bash\n    pinned:\n      - name: go\n        version: \"1.21.13\"\n",
        ));

        // Act
        develop(&m.deps(), false).unwrap();

        // Assert
        let persisted = m.repo.persisted_config().unwrap();
        let pinned = &persisted.dev_shell.package.pinned[0];
        assert_eq!(pinned.resolved_commit.as_deref(), Some("e607cb5"));
        assert_eq!(pinned.resolved_attr.as_deref(), Some("go_1_21"));
        assert!(m.writer.written().unwrap().contains("go_1_21"));
    }

    #[test]
    fn warns_about_empty_package_list_via_output_port() {
        // Arrange
        let m = Mocks::with_config(config_from_yaml("devShell:\n  package:\n    stable: []\n"));

        // Act
        develop(&m.deps(), false).unwrap();

        // Assert
        assert!(
            m.out
                .warns()
                .contains(&"No packages specified in lazynix.yaml".to_string())
        );
    }
}
