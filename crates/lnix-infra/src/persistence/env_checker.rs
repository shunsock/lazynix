//! Filesystem-backed [`EnvFilePresenceChecker`].

use std::path::PathBuf;

use lnix_domain::interface::persistence::EnvFilePresenceChecker;

use crate::paths::WorkspacePaths;

/// Checks dotenv file existence, resolving relative paths against the
/// config directory (matching how `nix develop` will resolve them).
pub struct FsEnvFileChecker {
    paths: WorkspacePaths,
}

impl FsEnvFileChecker {
    pub fn new(paths: WorkspacePaths) -> Self {
        Self { paths }
    }

    fn resolve(&self, path: &str) -> PathBuf {
        if path.starts_with('/') {
            PathBuf::from(path)
        } else {
            self.paths.config_dir().join(path.trim_start_matches("./"))
        }
    }
}

impl EnvFilePresenceChecker for FsEnvFileChecker {
    fn exists(&self, path: &str) -> bool {
        self.resolve(path).exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn resolves_relative_path_against_config_dir() {
        // Arrange
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join(".env"), "KEY=value").unwrap();
        let checker = FsEnvFileChecker::new(WorkspacePaths::new(dir.path()));

        // Act / Assert
        assert!(checker.exists("./.env"));
        assert!(checker.exists(".env"));
        assert!(!checker.exists(".env.missing"));
    }

    #[test]
    fn accepts_absolute_path_as_is() {
        // Arrange
        let dir = TempDir::new().unwrap();
        let file = dir.path().join(".env.global");
        fs::write(&file, "KEY=value").unwrap();
        let checker = FsEnvFileChecker::new(WorkspacePaths::new("./unrelated"));

        // Act / Assert
        assert!(checker.exists(file.to_str().unwrap()));
    }
}
