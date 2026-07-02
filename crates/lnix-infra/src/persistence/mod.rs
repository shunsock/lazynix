//! Filesystem adapters for the domain's persistence ports.

mod config_repository;
mod env_checker;
mod flake_writer;
mod scaffolder;

pub use config_repository::FsConfigRepository;
pub use env_checker::FsEnvFileChecker;
pub use flake_writer::FsFlakeWriter;
pub use scaffolder::FsProjectScaffolder;
