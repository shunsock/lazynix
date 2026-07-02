//! Value objects shared across the workspace.
//!
//! Each type wraps a `String` and enforces its invariant in
//! `TryFrom<String>`. Serde integration (`try_from` / `into`) ensures
//! that deserialized values are always valid.

mod env_var_name;
mod package_name;
mod package_version;
mod registry_url;
mod task_name;

pub use env_var_name::EnvVarName;
pub use package_name::PackageName;
pub use package_version::PackageVersion;
pub use registry_url::RegistryUrl;
pub use task_name::TaskName;
