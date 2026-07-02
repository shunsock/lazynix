//! Filesystem-backed [`ConfigRepository`].

use std::fs;

use lnix_domain::interface::persistence::ConfigRepository;
use lnix_domain::{Config, ConfigError, Settings};

use crate::paths::WorkspacePaths;

/// Reads and writes the config files under [`WorkspacePaths`].
pub struct FsConfigRepository {
    paths: WorkspacePaths,
}

impl FsConfigRepository {
    pub fn new(paths: WorkspacePaths) -> Self {
        Self { paths }
    }
}

impl ConfigRepository for FsConfigRepository {
    fn read_config(&self) -> Result<Config, ConfigError> {
        let path = self.paths.config_file();
        if !path.exists() {
            return Err(ConfigError::NotFound(
                self.paths.config_dir().display().to_string(),
            ));
        }
        let text = fs::read_to_string(&path)?;
        serde_yaml::from_str(&text).map_err(|e| ConfigError::Parse(e.to_string()))
    }

    fn write_config(&self, config: &Config) -> Result<(), ConfigError> {
        let text = serde_yaml::to_string(config).map_err(|e| ConfigError::Parse(e.to_string()))?;
        fs::write(self.paths.config_file(), text)?;
        Ok(())
    }

    fn read_settings(&self) -> Result<Option<Settings>, ConfigError> {
        let path = self.paths.settings_file();
        if !path.exists() {
            return Ok(None);
        }
        let text = fs::read_to_string(&path)?;
        let settings =
            serde_yaml::from_str(&text).map_err(|e| ConfigError::Parse(e.to_string()))?;
        Ok(Some(settings))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn repository_in(dir: &TempDir) -> FsConfigRepository {
        FsConfigRepository::new(WorkspacePaths::new(dir.path()))
    }

    #[test]
    fn reads_config_from_injected_directory() {
        // Arrange
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("lazynix.yaml"),
            "devShell:\n  package:\n    stable:\n      - name: python312\n",
        )
        .unwrap();

        // Act
        let config = repository_in(&dir).read_config().unwrap();

        // Assert
        assert_eq!(
            config.dev_shell.package.stable[0].name.as_str(),
            "python312"
        );
    }

    #[test]
    fn reports_missing_config_as_not_found() {
        // Arrange
        let dir = TempDir::new().unwrap();

        // Act
        let result = repository_in(&dir).read_config();

        // Assert
        assert!(matches!(result, Err(ConfigError::NotFound(_))));
    }

    #[test]
    fn reports_invalid_yaml_as_parse_error() {
        // Arrange
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("lazynix.yaml"), ":::not yaml:::").unwrap();

        // Act
        let result = repository_in(&dir).read_config();

        // Assert
        assert!(matches!(result, Err(ConfigError::Parse(_))));
    }

    #[test]
    fn round_trips_config_through_write() {
        // Arrange
        let dir = TempDir::new().unwrap();
        let repository = repository_in(&dir);
        fs::write(
            dir.path().join("lazynix.yaml"),
            "devShell:\n  package:\n    stable:\n      - name: bash\n",
        )
        .unwrap();
        let config = repository.read_config().unwrap();

        // Act
        repository.write_config(&config).unwrap();
        let reread = repository.read_config().unwrap();

        // Assert
        assert_eq!(reread.dev_shell.package.stable[0].name.as_str(), "bash");
    }

    #[test]
    fn missing_settings_is_none() {
        // Arrange
        let dir = TempDir::new().unwrap();

        // Act
        let settings = repository_in(&dir).read_settings().unwrap();

        // Assert
        assert!(settings.is_none());
    }

    #[test]
    fn reads_settings_when_present() {
        // Arrange
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("lazynix-settings.yaml"),
            "override-stable-package: \"github:NixOS/nixpkgs/nixos-25.06\"\n",
        )
        .unwrap();

        // Act
        let settings = repository_in(&dir).read_settings().unwrap();

        // Assert
        assert!(settings.unwrap().override_stable_package.is_some());
    }
}
