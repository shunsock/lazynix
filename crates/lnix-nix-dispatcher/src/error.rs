use thiserror::Error;

#[derive(Error, Debug)]
pub enum NixDispatcherError {
    #[error("Failed to execute nix command: {0}")]
    CommandExecution(String),

    #[error("Nix command exited with status {code}")]
    NonZeroExit { code: i32 },

    #[error("Failed to get exit code from command")]
    NoExitCode,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, NixDispatcherError>;
