//! Shared flake-generation pipeline used by `develop`, `test`, and `run`.
//!
//! These use-cases all read and validate the config, resolve pinned
//! versions, and render `flake.nix`. That common prefix lives here so
//! each use-case only adds its own tail (entering the shell, running
//! tests, executing a command).

use lnix_domain::{DevShellDefinition, render_flake};

use crate::deps::Deps;
use crate::error::ApplicationError;
use crate::event::UseCaseEvent;

/// A validated config together with the optional registry override that
/// settings supplied, ready to be rendered into a flake.
pub(crate) struct LoadedConfig {
    pub(crate) config: DevShellDefinition,
    override_url: Option<String>,
}

/// Reads, validates, and resolves the config — everything needed before
/// rendering, but without writing `flake.nix` yet.
pub(crate) fn load_config(deps: &Deps) -> Result<LoadedConfig, ApplicationError> {
    let settings = deps.repo.read_settings()?;
    let override_url = settings
        .and_then(|s| s.override_stable_package)
        .map(|url| url.as_str().to_string());

    deps.reporter.report(&UseCaseEvent::ReadingConfig);
    let mut config = deps.repo.read_config()?;

    deps.reporter.report(&UseCaseEvent::ValidatingConfig);
    for diagnostic in
        lnix_domain::validate_config(&config).map_err(lnix_domain::ConfigError::from)?
    {
        deps.reporter
            .report(&UseCaseEvent::ConfigDiagnostic(diagnostic.to_string()));
    }
    validate_env_files(deps, &config)?;

    resolve_pinned_packages(deps, &mut config)?;

    Ok(LoadedConfig {
        config,
        override_url,
    })
}

/// Fails when a dotenv file referenced by the config does not exist.
fn validate_env_files(deps: &Deps, config: &DevShellDefinition) -> Result<(), ApplicationError> {
    let Some(env) = &config.dev_shell.env else {
        return Ok(());
    };
    for dotenv_path in &env.dotenv {
        if !deps.env.exists(dotenv_path) {
            return Err(lnix_domain::ConfigError::DotenvFileNotFound(dotenv_path.clone()).into());
        }
    }
    Ok(())
}

/// Populates each pinned entry with its resolved `(commit, attr)`,
/// preferring the rendered `flake.nix` as the source of truth and
/// falling back to the version resolver for cache misses. Never
/// rewrites `lazynix.yaml`; the generated `flake.nix` and `flake.lock`
/// are the durable record.
// NOTE: a mid-loop resolver error leaves the caller's config partly
// mutated; safe today because load_config aborts on that error.
fn resolve_pinned_packages(
    deps: &Deps,
    config: &mut DevShellDefinition,
) -> Result<(), ApplicationError> {
    let pinned_keys: Vec<_> = config
        .dev_shell
        .package
        .pinned
        .iter()
        .map(|entry| (entry.name.clone(), entry.version.clone()))
        .collect();
    let mut cached = deps.flake_reader.read_pinned_inputs(&pinned_keys)?;
    let entries = config.dev_shell.package.pinned.iter_mut();
    for (entry, key) in entries.zip(pinned_keys) {
        if let Some(resolution) = cached.remove(&key) {
            entry.resolved_commit = Some(resolution.commit.to_string());
            entry.resolved_attr = Some(resolution.attr);
            continue;
        }
        deps.reporter.report(&UseCaseEvent::ResolvingPinned {
            name: entry.name.clone(),
            version: entry.version.clone(),
        });
        let resolved = deps.resolver.resolve(&entry.name, &entry.version)?;
        entry.resolved_commit = Some(resolved.commit.to_string());
        entry.resolved_attr = Some(resolved.attr);
    }
    Ok(())
}

/// Renders the loaded config and persists it as `flake.nix`.
pub(crate) fn write_flake(deps: &Deps, loaded: &LoadedConfig) -> Result<(), ApplicationError> {
    deps.reporter.report(&UseCaseEvent::GeneratingFlake);
    let contents = render_flake(&loaded.config, loaded.override_url.as_deref());
    deps.flake_writer.write_flake(&contents)?;
    deps.reporter.report(&UseCaseEvent::FlakeGenerated);
    Ok(())
}

/// Updates `flake.lock` when requested, or reports that it was skipped.
pub(crate) fn maybe_update_lock(deps: &Deps, update_lock: bool) -> Result<(), ApplicationError> {
    if update_lock {
        deps.reporter.report(&UseCaseEvent::UpdatingLock);
        deps.nix.flake_update()?;
        deps.reporter.report(&UseCaseEvent::LockUpdated);
    } else {
        deps.reporter.report(&UseCaseEvent::LockUpdateSkipped);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mocks::*;
    use lnix_domain::interface::persistence::{
        ConfigRepository, PinnedResolution, PinnedResolutions,
    };
    use std::collections::HashMap;

    const CACHED_SHA: &str = "5ed62751abf1dac19bf39d6ff8c07c8c37e81e0e";
    const RESOLVED_SHA: &str = "e607cb51abf1dac19bf39d6ff8c07c8c37e81e0e";
    const OVERRIDDEN_SHA: &str = "deafdeafdeafdeafdeafdeafdeafdeafdeafdeaf";

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

    fn resolutions_from(entries: &[(&str, &str, &str, &str)]) -> PinnedResolutions {
        let mut inputs: PinnedResolutions = HashMap::new();
        for (name, version, commit, attr) in entries {
            inputs.insert(
                (name.parse().unwrap(), version.parse().unwrap()),
                PinnedResolution {
                    commit: commit.parse().unwrap(),
                    attr: (*attr).to_string(),
                },
            );
        }
        inputs
    }

    #[test]
    fn hyphen_in_version_cache_hit_skips_resolver() {
        let m = Mocks::with_config(config_with_pinned(&[("go", "1.0.0-rc1")])).with_flake_reader(
            MockFlakeReader::new(resolutions_from(&[(
                "go",
                "1.0.0-rc1",
                CACHED_SHA,
                "go_1_21",
            )])),
        );
        let mut config = m.repo.read_config().unwrap();

        resolve_pinned_packages(&m.deps(), &mut config).unwrap();

        assert!(m.resolver.resolve_calls().is_empty());
        assert!(m.reporter.events().is_empty());
        let pinned = &config.dev_shell.package.pinned[0];
        assert_eq!(pinned.resolved_commit.as_deref(), Some(CACHED_SHA));
        assert_eq!(pinned.resolved_attr.as_deref(), Some("go_1_21"));
    }

    #[test]
    fn cache_miss_calls_resolver() {
        let m = Mocks::with_config(config_with_pinned(&[("go", "1.21.13")]))
            .with_flake_reader(MockFlakeReader::empty());
        let mut config = m.repo.read_config().unwrap();

        resolve_pinned_packages(&m.deps(), &mut config).unwrap();

        assert_eq!(m.resolver.resolve_calls(), vec!["go".to_string()]);
        let pinned = &config.dev_shell.package.pinned[0];
        assert_eq!(pinned.resolved_commit.as_deref(), Some(RESOLVED_SHA));
        assert_eq!(pinned.resolved_attr.as_deref(), Some("go_1_21"));
    }

    #[test]
    fn no_pinned_entries_no_resolver_calls() {
        let m = Mocks::with_config(config_from_yaml("devShell:\n  package:\n    stable: []\n"));
        let mut config = m.repo.read_config().unwrap();

        resolve_pinned_packages(&m.deps(), &mut config).unwrap();

        assert!(m.resolver.resolve_calls().is_empty());
    }

    #[test]
    fn flake_reader_error_propagates_as_application_error() {
        let m = Mocks::with_config(config_with_pinned(&[("go", "1.21.13")]))
            .with_failing_flake_reader();
        let mut config = m.repo.read_config().unwrap();

        let result = resolve_pinned_packages(&m.deps(), &mut config);

        assert!(matches!(
            result,
            Err(ApplicationError::Flake(lnix_domain::FlakeError::Read(_)))
        ));
        assert!(m.resolver.resolve_calls().is_empty());
    }

    #[test]
    fn already_resolved_inline_still_uses_cache() {
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable: []\n    pinned:\n      - name: go\n        version: \"1.21.13\"\n        resolvedCommit: OLD\n        resolvedAttr: OLD\n",
        ))
        .with_flake_reader(MockFlakeReader::new(resolutions_from(&[(
            "go",
            "1.21.13",
            OVERRIDDEN_SHA,
            "NEW_ATTR",
        )])));
        let mut config = m.repo.read_config().unwrap();

        resolve_pinned_packages(&m.deps(), &mut config).unwrap();

        assert!(m.resolver.resolve_calls().is_empty());
        let pinned = &config.dev_shell.package.pinned[0];
        assert_eq!(pinned.resolved_commit.as_deref(), Some(OVERRIDDEN_SHA));
        assert_eq!(pinned.resolved_attr.as_deref(), Some("NEW_ATTR"));
    }

    #[test]
    fn partial_cache_only_resolves_missing() {
        let m = Mocks::with_config(config_with_pinned(&[("go", "1.21.13"), ("rust", "1.70.0")]))
            .with_flake_reader(MockFlakeReader::new(resolutions_from(&[(
                "go",
                "1.21.13",
                CACHED_SHA,
                "go_1_21_cached",
            )])));
        let mut config = m.repo.read_config().unwrap();

        resolve_pinned_packages(&m.deps(), &mut config).unwrap();

        assert_eq!(m.resolver.resolve_calls(), vec!["rust".to_string()]);
        let go = &config.dev_shell.package.pinned[0];
        assert_eq!(go.resolved_commit.as_deref(), Some(CACHED_SHA));
        assert_eq!(go.resolved_attr.as_deref(), Some("go_1_21_cached"));
        let rust = &config.dev_shell.package.pinned[1];
        assert_eq!(rust.resolved_commit.as_deref(), Some(RESOLVED_SHA));
        assert_eq!(rust.resolved_attr.as_deref(), Some("go_1_21"));
    }

    #[test]
    fn passes_pinned_keys_matching_config_to_the_flake_reader() {
        let m = Mocks::with_config(config_with_pinned(&[("go", "1.21.13"), ("rust", "1.70.0")]));
        let mut config = m.repo.read_config().unwrap();

        resolve_pinned_packages(&m.deps(), &mut config).unwrap();

        let expected = vec![
            ("go".parse().unwrap(), "1.21.13".parse().unwrap()),
            ("rust".parse().unwrap(), "1.70.0".parse().unwrap()),
        ];
        assert_eq!(m.flake_reader.pinned_keys_calls(), vec![expected]);
    }
}
