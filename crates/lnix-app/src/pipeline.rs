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

/// Resolves pinned package versions and persists any newly resolved
/// entries back to `lazynix.yaml`.
fn resolve_pinned_packages(
    d: &Deps,
    config: &mut DevShellDefinition,
) -> Result<(), ApplicationError> {
    let mut resolved_any = false;
    for entry in &mut config.dev_shell.package.pinned {
        if entry.resolved_commit.is_some() && entry.resolved_attr.is_some() {
            continue;
        }
        d.out.info(&format!(
            "Resolving version for {} @ {}...",
            entry.name, entry.version
        ));
        let resolved = d.resolver.resolve(&entry.name, &entry.version)?;
        entry.resolved_commit = Some(resolved.commit);
        entry.resolved_attr = Some(resolved.attr);
        resolved_any = true;
    }

    if resolved_any {
        d.repo.write_config(config)?;
        d.out.info("Updated lazynix.yaml with resolved versions");
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
