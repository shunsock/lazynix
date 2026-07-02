//! Filesystem-backed [`FlakeWriter`].

use std::fs;

use lnix_domain::FlakeError;
use lnix_domain::interface::persistence::FlakeWriter;

use crate::paths::WorkspacePaths;

/// Writes `flake.nix` to the location owned by [`WorkspacePaths`].
pub struct FsFlakeWriter {
    paths: WorkspacePaths,
}

impl FsFlakeWriter {
    pub fn new(paths: WorkspacePaths) -> Self {
        Self { paths }
    }
}

impl FlakeWriter for FsFlakeWriter {
    fn write_flake(&self, contents: &str) -> Result<(), FlakeError> {
        fs::write(self.paths.flake_file(), contents)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn writes_flake_into_config_dir() {
        // Arrange
        let dir = TempDir::new().unwrap();
        let writer = FsFlakeWriter::new(WorkspacePaths::new(dir.path()));

        // Act
        writer.write_flake("{ description = \"test\"; }").unwrap();

        // Assert
        let written = fs::read_to_string(dir.path().join("flake.nix")).unwrap();
        assert_eq!(written, "{ description = \"test\"; }");
    }

    #[test]
    fn write_into_missing_directory_is_an_error() {
        // Arrange
        let writer = FsFlakeWriter::new(WorkspacePaths::new("./does-not-exist-xyz"));

        // Act
        let result = writer.write_flake("{}");

        // Assert
        assert!(matches!(result, Err(FlakeError::Write(_))));
    }
}
