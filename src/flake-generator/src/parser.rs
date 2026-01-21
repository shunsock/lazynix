use crate::config::Config;

use std::fs;
use std::path::PathBuf;

use crate::error::{FlakeGeneratorError, Result};

pub struct LazyNixParser {
    config_dir: PathBuf,
}

impl LazyNixParser {
    /// Create parser with explicit config directory
    pub fn new(config_dir: PathBuf) -> Self {
        Self { config_dir }
    }

    /// Get path to lazynix.yaml
    pub fn config_path(&self) -> PathBuf {
        self.config_dir.join("lazynix.yaml")
    }

    /// Read configuration from {config_dir}/lazynix.yaml
    pub fn read_config(&self) -> Result<Config> {
        let config_path = self.config_path();

        if !config_path.exists() {
            return Err(FlakeGeneratorError::ConfigNotFound);
        }

        let config_str = fs::read_to_string(&config_path)?;
        let config: Config = serde_yaml::from_str(&config_str)?;

        Ok(config)
    }

    /// Write config content to {config_dir}/lazynix.yaml
    pub fn write_config(&self, content: &str) -> Result<()> {
        // Validate config directory exists
        if !self.config_dir.exists() {
            return Err(FlakeGeneratorError::Validation(format!(
                "Config directory does not exist: {}. Please create it first.",
                self.config_dir.display()
            )));
        }

        let config_path = self.config_path();
        fs::write(config_path, content)?;
        Ok(())
    }
}

// Keep public function for backward compatibility
#[allow(dead_code)]
pub fn read_config() -> Result<Config> {
    LazyNixParser::new(PathBuf::from(".")).read_config()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;
    use tempfile::TempDir;

    /// Helper function to run tests in a temporary directory.
    ///
    /// Creates a temp dir, changes to it, runs the test closure,
    /// then restores the original directory and cleans up.
    fn with_temp_dir<F>(test_fn: F)
    where
        F: FnOnce(&TempDir),
    {
        let original_dir = env::current_dir().expect("Failed to get current directory");
        let temp_dir = TempDir::new().expect("Failed to create temp directory");

        env::set_current_dir(temp_dir.path()).expect("Failed to change to temp directory");

        // Run the test closure
        test_fn(&temp_dir);

        // Restore original directory
        // Note: TempDir will be automatically cleaned up when dropped
        env::set_current_dir(original_dir).expect("Failed to restore original directory");
    }

    #[test]
    fn test_parser_new() {
        let parser = LazyNixParser::new(PathBuf::from("./configs"));
        assert_eq!(parser.config_dir, PathBuf::from("./configs"));
    }

    #[test]
    fn test_config_path_getter() {
        let parser = LazyNixParser::new(PathBuf::from("./configs"));
        assert_eq!(
            parser.config_path(),
            PathBuf::from("./configs/lazynix.yaml")
        );
    }

    #[test]
    fn test_config_path_with_default() {
        let parser = LazyNixParser::new(PathBuf::from("."));
        assert_eq!(parser.config_path(), PathBuf::from("./lazynix.yaml"));
    }

    #[test]
    #[serial]
    fn test_parser_read_config_success() {
        with_temp_dir(|_temp_dir| {
            // Create config file
            let config_content = r#"
devShell:
  allowUnfree: false
  package:
    stable:
      - python312
"#;
            fs::write("lazynix.yaml", config_content).unwrap();

            // Read via struct with explicit path
            let parser = LazyNixParser::new(PathBuf::from("."));
            let config = parser.read_config().unwrap();

            assert_eq!(config.dev_shell.package.stable, vec!["python312"]);
        });
    }

    #[test]
    #[serial]
    fn test_parser_read_config_custom_dir() {
        with_temp_dir(|_temp_dir| {
            // Create configs subdirectory
            fs::create_dir("configs").unwrap();

            let config_content = r#"
devShell:
  allowUnfree: true
  package:
    stable:
      - rust
"#;
            fs::write("configs/lazynix.yaml", config_content).unwrap();

            // Read from custom directory
            let parser = LazyNixParser::new(PathBuf::from("./configs"));
            let config = parser.read_config().unwrap();

            assert!(config.dev_shell.allow_unfree);
            assert_eq!(config.dev_shell.package.stable, vec!["rust"]);
        });
    }

    #[test]
    #[serial]
    fn test_parser_read_config_not_found() {
        with_temp_dir(|_temp_dir| {
            // No config file exists
            let parser = LazyNixParser::new(PathBuf::from("."));
            let result = parser.read_config();

            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err(),
                FlakeGeneratorError::ConfigNotFound
            ));
        });
    }

    #[test]
    #[serial]
    fn test_read_config_backward_compatibility() {
        with_temp_dir(|_temp_dir| {
            // Create config file in current directory
            let config_content = r#"
devShell:
  allowUnfree: false
  package:
    stable:
      - gcc
"#;
            fs::write("lazynix.yaml", config_content).unwrap();

            // Public function should still work
            let config = read_config().unwrap();

            assert_eq!(config.dev_shell.package.stable, vec!["gcc"]);
        });
    }

    #[test]
    #[serial]
    fn test_parser_write_config_success() {
        with_temp_dir(|_temp_dir| {
            let parser = LazyNixParser::new(PathBuf::from("."));
            let content = "test content";

            parser.write_config(content).unwrap();

            let written = fs::read_to_string("lazynix.yaml").unwrap();
            assert_eq!(written, content);
        });
    }

    #[test]
    #[serial]
    fn test_parser_write_config_to_custom_dir() {
        with_temp_dir(|_temp_dir| {
            // Create custom directory
            fs::create_dir("configs").unwrap();

            let parser = LazyNixParser::new(PathBuf::from("./configs"));
            let content = "custom location";

            parser.write_config(content).unwrap();

            let written = fs::read_to_string("configs/lazynix.yaml").unwrap();
            assert_eq!(written, content);
        });
    }

    #[test]
    #[serial]
    fn test_parser_write_config_dir_not_exists() {
        with_temp_dir(|_temp_dir| {
            // Directory doesn't exist
            let parser = LazyNixParser::new(PathBuf::from("./nonexistent"));
            let content = "test";

            let result = parser.write_config(content);
            assert!(result.is_err());

            // Verify error message
            if let Err(FlakeGeneratorError::Validation(msg)) = result {
                assert!(msg.contains("does not exist"));
            } else {
                panic!("Expected Validation error");
            }
        });
    }
}
