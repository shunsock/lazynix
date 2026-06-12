//! Core domain types and value objects for LazyNix.
//!
//! This crate is the foundation of the workspace dependency graph.
//! It defines:
//!
//! - Value objects ([`PackageName`], [`TaskName`], ...) that validate
//!   their invariants at construction time, so that illegal values are
//!   unrepresentable everywhere downstream.
//! - The configuration AST ([`Config`] and friends) that mirrors the
//!   structure of `lazynix.yaml`.
//!
//! This crate performs no I/O and depends only on `serde` / `thiserror`.

mod config;
mod error;
mod values;

pub use config::{
    Config, DevShell, Env, EnvVar, Package, PackageEntry, PinnedPackageEntry, TaskDef,
    validate_config,
};
pub use error::{ParseError, ValidationError};
pub use values::{EnvVarName, PackageName, PackageVersion, RegistryUrl, TaskName};
