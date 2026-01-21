//! Nix command dispatcher for executing nix commands
//!
//! This crate provides functions to execute nix commands within the LazyNix environment.

pub mod commands;
pub mod error;

pub use commands::{
    run_flake_update, run_nix_develop, run_nix_develop_command, run_nix_test, run_task_in_nix_env,
};
pub use error::{NixDispatcherError, Result};
