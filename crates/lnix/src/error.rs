use thiserror::Error;

#[derive(Error, Debug)]
pub enum LazyNixError {
    #[error("File already exists: {0}. Use --force to overwrite")]
    FileExists(String),

    #[error("lazynix.yaml not found in current directory")]
    ConfigNotFound,

    #[error("Failed to read file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid YAML syntax: {0}")]
    YamlParse(#[from] serde_yaml::Error),

    #[error("Config validation failed: {0}")]
    Validation(String),

    #[allow(dead_code)]
    #[error("Settings validation failed: {0}")]
    SettingsValidation(String),

    #[error("Nix command failed: {0}")]
    NixCommand(String),

    #[allow(dead_code)]
    #[error("Dotenv file not found: {0}")]
    DotenvFileNotFound(String),

    #[allow(dead_code)]
    #[error(
        "Invalid environment variable name: {0}. Variable names must match [a-zA-Z_][a-zA-Z0-9_]*"
    )]
    InvalidEnvVarName(String),
}

pub type Result<T> = std::result::Result<T, LazyNixError>;

impl From<lnix_flake_generator::FlakeGeneratorError> for LazyNixError {
    fn from(err: lnix_flake_generator::FlakeGeneratorError) -> Self {
        match err {
            lnix_flake_generator::FlakeGeneratorError::Validation(msg) => {
                LazyNixError::Validation(msg)
            }
            lnix_flake_generator::FlakeGeneratorError::YamlParse(e) => LazyNixError::YamlParse(e),
            lnix_flake_generator::FlakeGeneratorError::IoError(e) => LazyNixError::IoError(e),
            lnix_flake_generator::FlakeGeneratorError::ConfigNotFound => {
                LazyNixError::ConfigNotFound
            }
        }
    }
}

impl From<lnix_nix_dispatcher::NixDispatcherError> for LazyNixError {
    fn from(err: lnix_nix_dispatcher::NixDispatcherError) -> Self {
        match err {
            lnix_nix_dispatcher::NixDispatcherError::CommandExecution(msg) => {
                LazyNixError::NixCommand(msg)
            }
            lnix_nix_dispatcher::NixDispatcherError::NonZeroExit { code } => {
                LazyNixError::NixCommand(format!("Nix command exited with status {}", code))
            }
            lnix_nix_dispatcher::NixDispatcherError::NoExitCode => {
                LazyNixError::NixCommand("Failed to get exit code".to_string())
            }
            lnix_nix_dispatcher::NixDispatcherError::Io(e) => LazyNixError::IoError(e),
        }
    }
}
