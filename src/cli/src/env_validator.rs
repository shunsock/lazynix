use crate::error::{LazyNixError, Result};
use lnix_flake_generator::Env;
use std::path::Path;

/// Check if an environment variable name is valid
/// Valid names must match the pattern: [a-zA-Z_][a-zA-Z0-9_]*
pub fn is_valid_env_var_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    let mut chars = name.chars();

    // First character must be a letter or underscore
    if let Some(first) = chars.next()
        && !first.is_ascii_alphabetic()
        && first != '_'
    {
        return false;
    }

    // Remaining characters must be alphanumeric or underscore
    for c in chars {
        if !c.is_ascii_alphanumeric() && c != '_' {
            return false;
        }
    }

    true
}

/// Resolve dotenv file path relative to base directory
fn resolve_dotenv_path(path: &str, base_dir: &Path) -> Result<std::path::PathBuf> {
    let resolved = if path.starts_with('/') {
        // Absolute path
        std::path::PathBuf::from(path)
    } else {
        // Relative path - resolve relative to base_dir
        base_dir.join(path.trim_start_matches("./"))
    };

    Ok(resolved)
}

/// Validate environment configuration
pub fn validate_env_config(env: &Option<Env>, base_dir: &Path) -> Result<()> {
    if let Some(env) = env {
        // Validate dotenv file existence
        for dotenv_path in &env.dotenv {
            let resolved_path = resolve_dotenv_path(dotenv_path, base_dir)?;
            if !resolved_path.exists() {
                return Err(LazyNixError::DotenvFileNotFound(format!(
                    "{} (resolved to {})",
                    dotenv_path,
                    resolved_path.display()
                )));
            }
        }

        // Validate environment variable names
        for envvar in &env.envvar {
            if !is_valid_env_var_name(&envvar.name) {
                return Err(LazyNixError::InvalidEnvVarName(envvar.name.clone()));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use lnix_flake_generator::EnvVar;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_is_valid_env_var_name() {
        // Valid names
        assert!(is_valid_env_var_name("MY_VAR"));
        assert!(is_valid_env_var_name("_PRIVATE"));
        assert!(is_valid_env_var_name("PATH"));
        assert!(is_valid_env_var_name("PYTHON_PATH"));
        assert!(is_valid_env_var_name("VAR123"));
        assert!(is_valid_env_var_name("_"));
        assert!(is_valid_env_var_name("a"));

        // Invalid names
        assert!(!is_valid_env_var_name("123VAR")); // Starts with number
        assert!(!is_valid_env_var_name("MY-VAR")); // Contains hyphen
        assert!(!is_valid_env_var_name("MY VAR")); // Contains space
        assert!(!is_valid_env_var_name("MY.VAR")); // Contains dot
        assert!(!is_valid_env_var_name("")); // Empty string
    }

    #[test]
    fn test_validate_env_dotenv_file_not_found() {
        let temp_dir = tempdir().unwrap();

        let env = Some(Env {
            dotenv: vec![".env.missing".to_string()],
            envvar: vec![],
        });

        let result = validate_env_config(&env, temp_dir.path());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LazyNixError::DotenvFileNotFound(_)
        ));
    }

    #[test]
    fn test_validate_env_invalid_var_name() {
        let temp_dir = tempdir().unwrap();

        let env = Some(Env {
            dotenv: vec![],
            envvar: vec![EnvVar {
                name: "123VAR".to_string(),
                value: "test".to_string(),
            }],
        });

        let result = validate_env_config(&env, temp_dir.path());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LazyNixError::InvalidEnvVarName(_)
        ));
    }

    #[test]
    fn test_validate_env_success() {
        let temp_dir = tempdir().unwrap();

        // Create a test .env file
        let env_file = temp_dir.path().join(".env");
        fs::write(&env_file, "TEST=value").unwrap();

        let env = Some(Env {
            dotenv: vec![".env".to_string()],
            envvar: vec![EnvVar {
                name: "MY_VAR".to_string(),
                value: "hello".to_string(),
            }],
        });

        let result = validate_env_config(&env, temp_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_env_with_absolute_path() {
        let temp_dir = tempdir().unwrap();

        // Create a test .env file with absolute path
        let env_file = temp_dir.path().join(".env.global");
        fs::write(&env_file, "TEST=value").unwrap();

        let env = Some(Env {
            dotenv: vec![env_file.to_str().unwrap().to_string()],
            envvar: vec![],
        });

        let result = validate_env_config(&env, temp_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_env_none() {
        let temp_dir = tempdir().unwrap();
        let result = validate_env_config(&None, temp_dir.path());
        assert!(result.is_ok());
    }
}
