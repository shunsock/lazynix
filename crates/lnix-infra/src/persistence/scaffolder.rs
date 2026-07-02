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
    use tempfile::TempDir;

    #[test]
    fn scaffolds_both_starter_files() {
        // Arrange
        let dir = TempDir::new().unwrap();
        let scaffolder = FsProjectScaffolder::new(WorkspacePaths::new(dir.path()));

        // Act
        scaffolder.write_config_template().unwrap();
        scaffolder.write_flake_template().unwrap();

        // Assert
        assert!(scaffolder.config_exists());
        assert!(scaffolder.flake_exists());
        let yaml = fs::read_to_string(dir.path().join("lazynix.yaml")).unwrap();
        assert!(yaml.contains("devShell"));
    }

    #[test]
    fn refuses_config_template_when_directory_is_missing() {
        // Arrange
        let scaffolder = FsProjectScaffolder::new(WorkspacePaths::new("./does-not-exist-xyz"));

        // Act
        let result = scaffolder.write_config_template();

        // Assert
        assert!(matches!(result, Err(ConfigError::NotFound(_))));
    }
}
