//! Repository ports for the project's own files
//! (`lazynix.yaml`, `lazynix-settings.yaml`, `flake.nix`, dotenv files).

mod config_repository;
mod env_file;
mod flake_writer;
mod scaffolder;

pub use config_repository::ConfigRepository;
pub use env_file::EnvFilePresenceChecker;
pub use flake_writer::FlakeWriter;
pub use scaffolder::ProjectScaffolder;
