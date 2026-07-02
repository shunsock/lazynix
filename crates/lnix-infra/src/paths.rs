//! Filesystem locations shared by the persistence adapters.

use std::path::{Path, PathBuf};

/// Where this workspace's files live.
///
/// Built once by the composition root and injected into each adapter,
/// so no adapter reads the current working directory. The flake is
/// anchored to the config directory on purpose: the previous behavior
/// wrote `flake.nix` to the CWD, which diverged from `--config-dir`
/// and scattered the two files across directories.
#[derive(Debug, Clone)]
pub struct WorkspacePaths {
    config_dir: PathBuf,
}

impl WorkspacePaths {
    /// Anchors every path to `config_dir`.
    pub fn new(config_dir: impl Into<PathBuf>) -> Self {
        Self {
            config_dir: config_dir.into(),
        }
    }

    /// The directory holding `lazynix.yaml` (and derived files).
    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    /// `{config_dir}/lazynix.yaml`
    pub fn config_file(&self) -> PathBuf {
        self.config_dir.join("lazynix.yaml")
    }

    /// `{config_dir}/lazynix-settings.yaml`
    pub fn settings_file(&self) -> PathBuf {
        self.config_dir.join("lazynix-settings.yaml")
    }

    /// `{config_dir}/flake.nix`
    pub fn flake_file(&self) -> PathBuf {
        self.config_dir.join("flake.nix")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchors_all_files_to_config_dir() {
        // Arrange
        let paths = WorkspacePaths::new("./configs");

        // Act / Assert
        assert_eq!(paths.config_file(), PathBuf::from("./configs/lazynix.yaml"));
        assert_eq!(
            paths.settings_file(),
            PathBuf::from("./configs/lazynix-settings.yaml")
        );
        assert_eq!(paths.flake_file(), PathBuf::from("./configs/flake.nix"));
    }
}
