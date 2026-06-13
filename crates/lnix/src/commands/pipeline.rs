//! Shared flake-generation pipeline used by `develop`, `test`, and `run`.
//!
//! These commands all read and validate the config, resolve pinned
//! versions, and render `flake.nix`. That common prefix lives here so
//! each command only adds its own tail (entering the shell, running
//! tests, executing a command).

use std::fs;
use std::path::Path;

use lnix_core::{Config, validate_config};
use lnix_flake_generator::{LazyNixParser, render_flake};
use lnix_nix_dispatcher::{resolve_version, run_flake_update};

use crate::env_validator;
use crate::error::Result;
use crate::lazynix_settings_yaml::Settings;

/// A validated config together with the optional registry override that
/// settings supplied, ready to be rendered into a flake.
pub struct LoadedConfig {
    pub config: Config,
    settings: Option<Settings>,
}

impl LoadedConfig {
    /// The stable nixpkgs override from settings, if any.
    fn override_url(&self) -> Option<&str> {
        self.settings
            .as_ref()
            .and_then(|s| s.override_stable_package.as_ref())
            .map(|url| url.as_str())
    }
}

/// Reads `lazynix-settings.yaml` if present.
///
/// `RegistryUrl` validates the override URL during deserialization, so
/// a returned `Settings` is always valid.
fn read_local_settings(config_dir: &Path) -> Result<Option<Settings>> {
    let settings_path = config_dir.join("lazynix-settings.yaml");
    if !settings_path.exists() {
        return Ok(None);
    }
    let settings_str = fs::read_to_string(settings_path)?;
    let settings: Settings = serde_yaml::from_str(&settings_str)?;
    Ok(Some(settings))
}

/// Resolves pinned package versions and persists any newly resolved
/// entries back to `lazynix.yaml`.
fn resolve_pinned_packages(parser: &LazyNixParser, config: &mut Config) -> Result<()> {
    let mut resolved_any = false;
    for entry in &mut config.dev_shell.package.pinned {
        if entry.resolved_commit.is_some() && entry.resolved_attr.is_some() {
            continue;
        }
        println!(
            "Resolving version for {} @ {}...",
            entry.name, entry.version
        );
        let resolved = resolve_version(entry.name.as_str(), entry.version.as_str())?;
        entry.resolved_commit = Some(resolved.commit);
        entry.resolved_attr = Some(resolved.attr);
        resolved_any = true;
    }

    if resolved_any {
        let yaml_content = serde_yaml::to_string(&config)?;
        parser.write_config(&yaml_content)?;
        println!("Updated lazynix.yaml with resolved versions");
    }
    Ok(())
}

/// Reads, validates, and resolves the config — everything needed before
/// rendering, but without writing `flake.nix` yet.
pub fn load_config(config_dir: &Path) -> Result<LoadedConfig> {
    let settings = read_local_settings(config_dir)?;

    println!("Reading configuration from {}...", config_dir.display());
    let parser = LazyNixParser::new(config_dir.to_path_buf());
    let mut config = parser.read_config()?;

    println!("Validating configuration...");
    validate_config(&config)?;
    env_validator::validate_env_config(&config.dev_shell.env, config_dir)?;

    resolve_pinned_packages(&parser, &mut config)?;

    Ok(LoadedConfig { config, settings })
}

/// Renders the loaded config to `./flake.nix`.
pub fn write_flake(loaded: &LoadedConfig) -> Result<()> {
    println!("Generating flake.nix...");
    let flake_content = render_flake(&loaded.config, loaded.override_url());
    fs::write("flake.nix", flake_content)?;
    println!("✓ flake.nix generated successfully");
    Ok(())
}

/// Updates `flake.lock` when requested, or reports that it was skipped.
pub fn maybe_update_lock(update_lock: bool) -> Result<()> {
    if update_lock {
        run_flake_update()?;
    } else {
        println!("Skipping flake.lock update (use --update to update)");
    }
    Ok(())
}
