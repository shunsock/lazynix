//! Configuration AST mirroring the structure of `lazynix.yaml`.
//!
//! Field-level invariants (package names, task names, ...) are enforced
//! by the value objects in [`crate::values`] during deserialization.
//! Cross-field constraints are checked by [`validate_config`].

mod dev_shell;
mod env;
mod package;
mod settings;
mod task;
mod validate;

pub use dev_shell::{Config, DevShell};
pub use env::{Env, EnvVar};
pub use package::{Package, PackageEntry, PinnedPackageEntry};
pub use settings::Settings;
pub use task::TaskDef;
pub use validate::validate_config;
