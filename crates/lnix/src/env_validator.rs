use crate::error::{LazyNixError, Result};
use lnix_core::Env;
use std::path::{Path, PathBuf};

/// Resolve dotenv file path relative to base directory
fn resolve_dotenv_path(path: &str, base_dir: &Path) -> PathBuf {
    if path.starts_with('/') {
        // Absolute path
        PathBuf::from(path)
    } else {
        // Relative path - resolve relative to base_dir
        base_dir.join(path.trim_start_matches("./"))
    }
}

/// Validate environment configuration.
///
/// Variable name validity is already guaranteed by
/// [`lnix_core::EnvVarName`]; only filesystem state (dotenv file
/// existence) needs to be checked here.
pub fn validate_env_config(env: &Option<Env>, base_dir: &Path) -> Result<()> {
    let Some(env) = env else {
        return Ok(());
    };

    for dotenv_path in &env.dotenv {
        let resolved_path = resolve_dotenv_path(dotenv_path, base_dir);
        if !resolved_path.exists() {
            return Err(LazyNixError::DotenvFileNotFound(format!(
                "{} (resolved to {})",
                dotenv_path,
                resolved_path.display()
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn rejects_missing_dotenv_file() {
        // Arrange
        let temp_dir = tempdir().unwrap();
        let env: Option<Env> = serde_yaml::from_str("dotenv:\n  - .env.missing\n").unwrap();

        // Act
        let result = validate_env_config(&env, temp_dir.path());

        // Assert
        assert!(matches!(
            result.unwrap_err(),
            LazyNixError::DotenvFileNotFound(_)
        ));
    }

    #[test]
    fn accepts_existing_dotenv_with_relative_path() {
        // Arrange
        let temp_dir = tempdir().unwrap();
        fs::write(temp_dir.path().join(".env"), "TEST=value").unwrap();
        let env: Option<Env> = serde_yaml::from_str(
            "dotenv:\n  - .env\nenvvar:\n  - name: MY_VAR\n    value: hello\n",
        )
        .unwrap();

        // Act
        let result = validate_env_config(&env, temp_dir.path());

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn accepts_existing_dotenv_with_absolute_path() {
        // Arrange
        let temp_dir = tempdir().unwrap();
        let env_file = temp_dir.path().join(".env.global");
        fs::write(&env_file, "TEST=value").unwrap();
        let yaml = format!("dotenv:\n  - {}\n", env_file.display());
        let env: Option<Env> = serde_yaml::from_str(&yaml).unwrap();

        // Act
        let result = validate_env_config(&env, temp_dir.path());

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn accepts_absent_env_section() {
        // Arrange
        let temp_dir = tempdir().unwrap();

        // Act
        let result = validate_env_config(&None, temp_dir.path());

        // Assert
        assert!(result.is_ok());
    }
}
