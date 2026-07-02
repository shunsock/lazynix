//! Package-lint domain logic.
//!
//! Classification of `nix eval` failures, the aggregate validation
//! result, and user-facing report assembly. Executing `nix eval`
//! itself is I/O and lives behind
//! [`crate::interface::gateway::NixEvaluator`].

mod classification;
mod error;
mod report;
mod result;

pub use classification::classify_nix_eval_error;
pub use error::PackageValidationError;
pub use report::{format_validation_result, format_validation_result_verbose};
pub use result::ValidationResult;
