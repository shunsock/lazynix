//! Nix flake generator for LazyNix

mod config;
mod error;
mod generator;
mod package_validator;
mod parser;

// Public API
pub use config::{validate_config, Config, DevShell, Env, EnvVar, Package, TaskDef};
pub use error::{FlakeGeneratorError, Result};
pub use generator::render_flake;
pub use parser::LazyNixParser;
