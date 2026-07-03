//! Domain layer of LazyNix: the innermost crate of the workspace.
//!
//! It defines:
//!
//! - Value objects ([`PackageName`], [`TaskName`], ...) that validate
//!   their invariants at construction time, so that illegal values are
//!   unrepresentable everywhere downstream.
//! - The configuration AST ([`DevShellDefinition`] and friends) that mirrors the
//!   structure of `lazynix.yaml`.
//! - Pure domain services ([`service`]): flake rendering, lint
//!   classification and reporting, task-command interpolation.
//! - Ports ([`interface`]): the traits infrastructure adapters
//!   implement, together with the focused error types they return.
//!
//! This crate performs no I/O and depends only on `serde` / `thiserror`.

mod definition;
mod error;
mod values;

pub mod interface;
pub mod service;

pub use definition::{
    DevShell, DevShellDefinition, Env, EnvVar, Package, PackageEntry, PinnedPackageEntry, Settings,
    TaskDef, validate_config,
};
pub use error::{ConfigError, Diagnostic, FlakeError, NixError, ParseError, ValidationError};
pub use service::flake::render_flake;
pub use service::lint::{
    PackageValidationError, ValidationResult, classify_nix_eval_error, format_validation_result,
    format_validation_result_verbose,
};
pub use service::task::interpolate_command;
pub use values::{EnvVarName, PackageName, PackageVersion, RegistryUrl, TaskName};
