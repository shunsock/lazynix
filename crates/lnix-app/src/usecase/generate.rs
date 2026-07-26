//! `lnix generate` — render `flake.nix` from `lazynix.yaml` and exit.

use crate::deps::Deps;
use crate::error::ApplicationError;
use crate::pipeline;

/// Renders `flake.nix` from the current config and returns the exit code.
///
/// Unlike `develop`, `generate` never spawns any Nix subprocess: no
/// shell entry, no ad-hoc command, and no `flake.lock` update. This
/// keeps the command usable in environments (CI dry runs, editor
/// integrations) where evaluating Nix is undesirable, and preserves
/// the property that a config without `pinned` entries requires zero
/// Nix invocations.
pub fn generate(d: &Deps) -> Result<i32, ApplicationError> {
    let loaded = pipeline::load_config(d)?;
    pipeline::write_flake(d, &loaded)?;
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mocks::*;
    use lnix_domain::ConfigError;

    #[test]
    fn writes_flake_with_configured_package_and_returns_zero() {
        // Arrange
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable:\n      - name: bash\n",
        ));

        // Act
        let code = generate(&m.deps()).unwrap();

        // Assert
        assert_eq!(code, 0);
        let written = m.writer.written().expect("flake.nix should be written");
        assert!(written.contains("bash"));
    }

    #[test]
    fn does_not_enter_the_dev_shell() {
        // Arrange
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable:\n      - name: bash\n",
        ));

        // Act
        generate(&m.deps()).unwrap();

        // Assert
        assert_eq!(m.nix.develop_calls(), 0);
    }

    #[test]
    fn does_not_execute_any_ad_hoc_command() {
        // Arrange
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable:\n      - name: bash\n",
        ));

        // Act
        generate(&m.deps()).unwrap();

        // Assert
        assert!(m.nix.develop_command_args().is_none());
    }

    #[test]
    fn does_not_update_the_flake_lock() {
        // Arrange
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable:\n      - name: bash\n",
        ));

        // Act
        generate(&m.deps()).unwrap();

        // Assert
        assert_eq!(m.nix.flake_update_calls(), 0);
    }

    #[test]
    fn missing_config_short_circuits_before_any_side_effect() {
        // Arrange
        let m = Mocks::with_missing_config();

        // Act
        let result = generate(&m.deps());

        // Assert
        assert!(matches!(
            result,
            Err(ApplicationError::Config(ConfigError::NotFound(_)))
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
        generate(&m.deps()).unwrap();

        // Assert
        let persisted = m.repo.persisted_config().unwrap();
        let pinned = &persisted.dev_shell.package.pinned[0];
        assert_eq!(pinned.resolved_commit.as_deref(), Some("e607cb5"));
        assert_eq!(pinned.resolved_attr.as_deref(), Some("go_1_21"));
        assert!(m.writer.written().unwrap().contains("go_1_21"));
    }

    #[test]
    fn missing_dotenv_fails_before_writing_flake() {
        // Arrange
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable:\n      - name: bash\n  env:\n    dotenv:\n      - .env\n",
        ))
        .with_missing_env_files();

        // Act
        let result = generate(&m.deps());

        // Assert
        assert!(matches!(
            result,
            Err(ApplicationError::Config(ConfigError::DotenvFileNotFound(_)))
        ));
        assert!(m.writer.written().is_none());
    }
}
