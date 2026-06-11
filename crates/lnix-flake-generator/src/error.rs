use thiserror::Error;

#[derive(Error, Debug)]
pub enum FlakeGeneratorError {
    #[error("Config validation failed: {0}")]
    Validation(String),

    #[error("Invalid YAML syntax: {0}")]
    YamlParse(#[from] serde_yaml::Error),

    #[error("Failed to read file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Config file not found")]
    ConfigNotFound,
}

pub type Result<T> = std::result::Result<T, FlakeGeneratorError>;
