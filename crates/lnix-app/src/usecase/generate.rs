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
    use lnix_domain::interface::persistence::{PinnedResolution, PinnedResolutions};
    use std::collections::HashMap;

    #[test]
    fn writes_flake_with_configured_package_and_returns_zero() {
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable:\n      - name: bash\n",
        ));

        let code = generate(&m.deps()).unwrap();

        assert_eq!(code, 0);
        let written = m
            .flake_writer
            .written()
            .expect("flake.nix should be written");
        assert!(written.contains("bash"));
    }

    #[test]
    fn does_not_enter_the_dev_shell() {
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable:\n      - name: bash\n",
        ));

        generate(&m.deps()).unwrap();

        assert_eq!(m.nix.develop_calls(), 0);
    }

    #[test]
    fn does_not_execute_any_ad_hoc_command() {
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable:\n      - name: bash\n",
        ));

        generate(&m.deps()).unwrap();

        assert!(m.nix.develop_command_args().is_none());
    }

    #[test]
    fn does_not_update_the_flake_lock() {
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable:\n      - name: bash\n",
        ));

        generate(&m.deps()).unwrap();

        assert_eq!(m.nix.flake_update_calls(), 0);
    }

    #[test]
    fn missing_config_short_circuits_before_any_side_effect() {
        let m = Mocks::with_missing_config();

        let result = generate(&m.deps());

        assert!(matches!(
            result,
            Err(ApplicationError::Config(ConfigError::NotFound(_)))
        ));
        assert!(m.flake_writer.written().is_none());
    }

    #[test]
    fn resolves_pinned_packages_and_renders_them_into_flake() {
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable:\n      - name: bash\n    pinned:\n      - name: go\n        version: \"1.21.13\"\n",
        ));

        generate(&m.deps()).unwrap();

        let written = m
            .flake_writer
            .written()
            .expect("flake.nix should be written");
        assert!(written.contains("e607cb5"));
        assert!(written.contains("go_1_21"));
    }

    #[test]
    fn generate_does_not_write_back_lazynix_yaml_when_pinned_resolves() {
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable: []\n    pinned:\n      - name: go\n        version: \"1.21.13\"\n",
        ))
        .with_flake_reader(MockFlakeReader::empty());

        generate(&m.deps()).unwrap();

        assert!(m.flake_writer.written().is_some());
    }

    #[test]
    fn generate_reuses_flake_reader_cache_and_skips_resolver() {
        let mut cached: PinnedResolutions = HashMap::new();
        cached.insert(
            ("go".parse().unwrap(), "1.21.13".parse().unwrap()),
            PinnedResolution {
                commit: "CACHED_COMMIT".to_string(),
                attr: "CACHED_ATTR".to_string(),
            },
        );
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable: []\n    pinned:\n      - name: go\n        version: \"1.21.13\"\n",
        ))
        .with_flake_reader(MockFlakeReader::new(cached));

        generate(&m.deps()).unwrap();

        assert!(m.resolver.resolve_calls().is_empty());
        let written = m
            .flake_writer
            .written()
            .expect("flake.nix should be written");
        assert!(written.contains("CACHED_COMMIT"));
        assert!(written.contains("CACHED_ATTR"));
    }

    #[test]
    fn missing_dotenv_fails_before_writing_flake() {
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable:\n      - name: bash\n  env:\n    dotenv:\n      - .env\n",
        ))
        .with_missing_env_files();

        let result = generate(&m.deps());

        assert!(matches!(
            result,
            Err(ApplicationError::Config(ConfigError::DotenvFileNotFound(_)))
        ));
        assert!(m.flake_writer.written().is_none());
    }
}
