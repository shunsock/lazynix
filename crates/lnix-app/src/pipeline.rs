//! Shared flake-generation pipeline used by `develop`, `test`, and `run`.
//!
//! These use-cases all read and validate the config, resolve pinned
//! versions, and render `flake.nix`. That common prefix lives here so
//! each use-case only adds its own tail (entering the shell, running
//! tests, executing a command).

use lnix_domain::{DevShellDefinition, render_flake};

use crate::deps::Deps;
use crate::error::ApplicationError;

/// A validated config together with the optional registry override that
/// settings supplied, ready to be rendered into a flake.
pub(crate) struct LoadedConfig {
    pub(crate) config: DevShellDefinition,
    override_url: Option<String>,
}

/// Reads, validates, and resolves the config — everything needed before
/// rendering, but without writing `flake.nix` yet.
pub(crate) fn load_config(d: &Deps) -> Result<LoadedConfig, ApplicationError> {
    let settings = d.repo.read_settings()?;
    let override_url = settings
        .and_then(|s| s.override_stable_package)
        .map(|url| url.as_str().to_string());

    d.out.info("Reading configuration...");
    let mut config = d.repo.read_config()?;

    d.out.info("Validating configuration...");
    for diagnostic in
        lnix_domain::validate_config(&config).map_err(lnix_domain::ConfigError::from)?
    {
        d.out.warn(&diagnostic.to_string());
    }
    validate_env_files(d, &config)?;

    resolve_pinned_packages(d, &mut config)?;

    Ok(LoadedConfig {
        config,
        override_url,
    })
}

/// Fails when a dotenv file referenced by the config does not exist.
fn validate_env_files(d: &Deps, config: &DevShellDefinition) -> Result<(), ApplicationError> {
    let Some(env) = &config.dev_shell.env else {
        return Ok(());
    };
    for dotenv_path in &env.dotenv {
        if !d.env.exists(dotenv_path) {
            return Err(lnix_domain::ConfigError::DotenvFileNotFound(dotenv_path.clone()).into());
        }
    }
    Ok(())
}

/// Populates each pinned entry with its resolved `(commit, attr)`,
/// preferring the rendered `flake.nix` as the source of truth and
/// falling back to the version resolver for cache misses. Never
/// rewrites `lazynix.yaml`; the flake and lockfile are the durable
/// record.
fn resolve_pinned_packages(
    d: &Deps,
    config: &mut DevShellDefinition,
) -> Result<(), ApplicationError> {
    let cached = d.flake_reader.read_pinned_inputs()?;
    for entry in &mut config.dev_shell.package.pinned {
        let key = (entry.name.clone(), entry.version.clone());
        if let Some((commit, attr)) = cached.get(&key) {
            entry.resolved_commit = Some(commit.clone());
            entry.resolved_attr = Some(attr.clone());
            continue;
        }
        d.out.info(&format!(
            "Resolving version for {} @ {}...",
            entry.name, entry.version
        ));
        let resolved = d.resolver.resolve(&entry.name, &entry.version)?;
        entry.resolved_commit = Some(resolved.commit);
        entry.resolved_attr = Some(resolved.attr);
    }
    Ok(())
}

/// Renders the loaded config and persists it as `flake.nix`.
pub(crate) fn write_flake(d: &Deps, loaded: &LoadedConfig) -> Result<(), ApplicationError> {
    d.out.info("Generating flake.nix...");
    let contents = render_flake(&loaded.config, loaded.override_url.as_deref());
    d.writer.write_flake(&contents)?;
    d.out.info("✓ flake.nix generated successfully");
    Ok(())
}

/// Updates `flake.lock` when requested, or reports that it was skipped.
pub(crate) fn maybe_update_lock(d: &Deps, update_lock: bool) -> Result<(), ApplicationError> {
    if update_lock {
        d.out.info("Updating flake.lock...");
        d.nix.flake_update()?;
        d.out.info("flake.lock updated successfully");
    } else {
        d.out
            .info("Skipping flake.lock update (use --update to update)");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mocks::*;
    use lnix_domain::interface::persistence::{ConfigRepository, PinnedResolutions};
    use std::collections::HashMap;

    fn config_with_pinned(entries: &[(&str, &str)]) -> DevShellDefinition {
        let mut yaml = String::from("devShell:\n  package:\n    stable: []\n    pinned:\n");
        for (name, version) in entries {
            yaml.push_str(&format!(
                "      - name: {}\n        version: \"{}\"\n",
                name, version
            ));
        }
        config_from_yaml(&yaml)
    }

    fn cache(entries: &[(&str, &str, &str, &str)]) -> PinnedResolutions {
        let mut inputs: PinnedResolutions = HashMap::new();
        for (name, version, commit, attr) in entries {
            inputs.insert(
                (name.parse().unwrap(), version.parse().unwrap()),
                ((*commit).to_string(), (*attr).to_string()),
            );
        }
        inputs
    }

    #[test]
    fn cache_hit_skips_resolver() {
        let m = Mocks::with_config(config_with_pinned(&[("go", "1.21.13")])).with_flake_reader(
            MockFlakeReader::new(cache(&[("go", "1.21.13", "5ed6275", "go_1_21")])),
        );
        let mut config = m.repo.read_config().unwrap();

        resolve_pinned_packages(&m.deps(), &mut config).unwrap();

        assert!(m.resolver.resolve_calls().is_empty());
        let pinned = &config.dev_shell.package.pinned[0];
        assert_eq!(pinned.resolved_commit.as_deref(), Some("5ed6275"));
        assert_eq!(pinned.resolved_attr.as_deref(), Some("go_1_21"));
        assert!(m.repo.persisted_config().is_none());
    }

    #[test]
    fn cache_miss_calls_resolver() {
        let m = Mocks::with_config(config_with_pinned(&[("go", "1.21.13")]))
            .with_flake_reader(MockFlakeReader::empty());
        let mut config = m.repo.read_config().unwrap();

        resolve_pinned_packages(&m.deps(), &mut config).unwrap();

        assert_eq!(m.resolver.resolve_calls(), vec!["go".to_string()]);
        let pinned = &config.dev_shell.package.pinned[0];
        assert_eq!(pinned.resolved_commit.as_deref(), Some("e607cb5"));
        assert_eq!(pinned.resolved_attr.as_deref(), Some("go_1_21"));
        assert!(m.repo.persisted_config().is_none());
    }

    #[test]
    fn no_pinned_entries_no_resolver_calls() {
        let m = Mocks::with_config(config_from_yaml("devShell:\n  package:\n    stable: []\n"));
        let mut config = m.repo.read_config().unwrap();

        resolve_pinned_packages(&m.deps(), &mut config).unwrap();

        assert!(m.resolver.resolve_calls().is_empty());
        assert!(m.repo.persisted_config().is_none());
    }

    #[test]
    fn flake_reader_error_propagates_as_application_error() {
        let m = Mocks::with_config(config_with_pinned(&[("go", "1.21.13")]))
            .with_flake_reader(MockFlakeReader::failing());
        let mut config = m.repo.read_config().unwrap();

        let result = resolve_pinned_packages(&m.deps(), &mut config);

        assert!(matches!(
            result,
            Err(ApplicationError::Flake(lnix_domain::FlakeError::Read(_)))
        ));
        assert!(m.resolver.resolve_calls().is_empty());
        assert!(m.repo.persisted_config().is_none());
    }

    #[test]
    fn already_resolved_inline_still_uses_cache() {
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable: []\n    pinned:\n      - name: go\n        version: \"1.21.13\"\n        resolvedCommit: OLD\n        resolvedAttr: OLD\n",
        ))
        .with_flake_reader(MockFlakeReader::new(cache(&[(
            "go", "1.21.13", "NEW_COMMIT", "NEW_ATTR",
        )])));
        let mut config = m.repo.read_config().unwrap();

        resolve_pinned_packages(&m.deps(), &mut config).unwrap();

        assert!(m.resolver.resolve_calls().is_empty());
        let pinned = &config.dev_shell.package.pinned[0];
        assert_eq!(pinned.resolved_commit.as_deref(), Some("NEW_COMMIT"));
        assert_eq!(pinned.resolved_attr.as_deref(), Some("NEW_ATTR"));
        assert!(m.repo.persisted_config().is_none());
    }

    #[test]
    fn partial_cache_only_resolves_missing() {
        let m = Mocks::with_config(config_with_pinned(&[("go", "1.21.13"), ("rust", "1.70.0")]))
            .with_flake_reader(MockFlakeReader::new(cache(&[(
                "go",
                "1.21.13",
                "5ed6275",
                "go_1_21_cached",
            )])));
        let mut config = m.repo.read_config().unwrap();

        resolve_pinned_packages(&m.deps(), &mut config).unwrap();

        assert_eq!(m.resolver.resolve_calls(), vec!["rust".to_string()]);
        let go = &config.dev_shell.package.pinned[0];
        assert_eq!(go.resolved_commit.as_deref(), Some("5ed6275"));
        assert_eq!(go.resolved_attr.as_deref(), Some("go_1_21_cached"));
        let rust = &config.dev_shell.package.pinned[1];
        assert_eq!(rust.resolved_commit.as_deref(), Some("e607cb5"));
        assert_eq!(rust.resolved_attr.as_deref(), Some("go_1_21"));
        assert!(m.repo.persisted_config().is_none());
    }
}
