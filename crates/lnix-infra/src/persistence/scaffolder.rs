//! Filesystem-backed [`ProjectScaffolder`] with bundled templates.

use std::fs;

use lnix_domain::interface::persistence::ProjectScaffolder;
use lnix_domain::{ConfigError, FlakeError};

use crate::paths::WorkspacePaths;

const YAML_TEMPLATE: &str = include_str!("../../templates/lazynix.yaml.template");
const FLAKE_TEMPLATE: &str = include_str!("../../templates/flake.nix.init.template");

/// Writes the bundled starter files into the workspace.
pub struct FsProjectScaffolder {
    paths: WorkspacePaths,
}

impl FsProjectScaffolder {
    pub fn new(paths: WorkspacePaths) -> Self {
        Self { paths }
    }
}

impl ProjectScaffolder for FsProjectScaffolder {
    fn config_exists(&self) -> bool {
        self.paths.config_file().exists()
    }

    fn flake_exists(&self) -> bool {
        self.paths.flake_file().exists()
    }

    fn config_path_display(&self) -> String {
        self.paths.config_file().display().to_string()
    }

    fn flake_path_display(&self) -> String {
        self.paths.flake_file().display().to_string()
    }

    fn write_config_template(&self) -> Result<(), ConfigError> {
        if !self.paths.config_dir().exists() {
            return Err(ConfigError::NotFound(
                self.paths.config_dir().display().to_string(),
            ));
        }
        fs::write(self.paths.config_file(), YAML_TEMPLATE)?;
        Ok(())
    }

    fn write_flake_template(&self) -> Result<(), FlakeError> {
        fs::write(self.paths.flake_file(), FLAKE_TEMPLATE)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lnix_domain::DevShellDefinition;
    use tempfile::TempDir;

    #[test]
    fn scaffolds_both_starter_files() {
        let dir = TempDir::new().unwrap();
        let scaffolder = FsProjectScaffolder::new(WorkspacePaths::new(dir.path()));

        scaffolder.write_config_template().unwrap();
        scaffolder.write_flake_template().unwrap();

        assert!(scaffolder.config_exists());
        assert!(scaffolder.flake_exists());
        let yaml = fs::read_to_string(dir.path().join("lazynix.yaml")).unwrap();
        assert!(yaml.contains("devShell"));
    }

    #[test]
    fn refuses_config_template_when_directory_is_missing() {
        let scaffolder = FsProjectScaffolder::new(WorkspacePaths::new("./does-not-exist-xyz"));

        let result = scaffolder.write_config_template();

        assert!(matches!(result, Err(ConfigError::NotFound(_))));
    }

    #[test]
    fn config_template_ships_commented_pinned_example() {
        let dir = TempDir::new().unwrap();
        let scaffolder = FsProjectScaffolder::new(WorkspacePaths::new(dir.path()));

        scaffolder.write_config_template().unwrap();
        let yaml = fs::read_to_string(dir.path().join("lazynix.yaml")).unwrap();

        assert!(
            yaml.contains("# pinned:"),
            "expected commented `pinned:` heading, got:\n{yaml}"
        );
        assert!(
            yaml.contains("#   - name: go"),
            "expected commented pinned entry name, got:\n{yaml}"
        );
        assert!(
            yaml.contains("#     version: \"1.21.13\""),
            "expected commented pinned entry version, got:\n{yaml}"
        );
        assert!(
            yaml.contains("lnix search"),
            "expected pinned example to reference `lnix search`, got:\n{yaml}"
        );
    }

    #[test]
    fn config_template_parses_with_empty_pinned() {
        let dir = TempDir::new().unwrap();
        let scaffolder = FsProjectScaffolder::new(WorkspacePaths::new(dir.path()));

        scaffolder.write_config_template().unwrap();
        let yaml = fs::read_to_string(dir.path().join("lazynix.yaml")).unwrap();
        let config: DevShellDefinition = serde_yaml::from_str(&yaml).unwrap();

        assert!(
            config.dev_shell.package.pinned.is_empty(),
            "`lnix init` must emit an empty pinned array; found {} entries",
            config.dev_shell.package.pinned.len()
        );
    }

    #[test]
    fn uncommenting_pinned_example_yields_valid_config() {
        let dir = TempDir::new().unwrap();
        let scaffolder = FsProjectScaffolder::new(WorkspacePaths::new(dir.path()));
        scaffolder.write_config_template().unwrap();
        let yaml = fs::read_to_string(dir.path().join("lazynix.yaml")).unwrap();

        let opted_in = simulate_pinned_opt_in(&yaml);
        let config: DevShellDefinition = serde_yaml::from_str(&opted_in).unwrap_or_else(|e| {
            panic!("uncommented template must parse; error={e}\nyaml:\n{opted_in}")
        });

        let pinned = &config.dev_shell.package.pinned;
        assert_eq!(pinned.len(), 1, "yaml:\n{opted_in}");
        assert_eq!(pinned[0].name.as_str(), "go");
        assert_eq!(pinned[0].version.as_str(), "1.21.13");
    }

    // SEE: crates/lnix-infra/templates/lazynix.yaml.template
    fn simulate_pinned_opt_in(template: &str) -> String {
        template
            .lines()
            .map(uncomment_pinned_example_line)
            .collect::<Vec<_>>()
            .join("\n")
    }

    // SEE: crates/lnix-infra/templates/lazynix.yaml.template
    fn uncomment_pinned_example_line(line: &str) -> String {
        let indent: String = line.chars().take_while(|c| c.is_whitespace()).collect();
        let rest = &line[indent.len()..];
        let Some(after_hash) = rest.strip_prefix("# ") else {
            return line.to_string();
        };
        if is_pinned_example_line(after_hash) {
            format!("{indent}{after_hash}")
        } else {
            line.to_string()
        }
    }

    fn is_pinned_example_line(after_hash: &str) -> bool {
        after_hash.starts_with("pinned:")
            || after_hash.starts_with("  - name: go")
            || after_hash.starts_with("    version:")
    }
}
