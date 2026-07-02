//! LazyNix Linter Library
//!
//! This library provides package validation functionality using `nix eval`
//! to check package existence and architecture compatibility before compilation.
//!
//! Pure lint logic (error classification, result type, report
//! formatting) moved to `lnix_domain`; the re-exports below keep this
//! crate's public API stable until the crate is dismantled.

#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

pub mod error;
pub mod nix_eval;
pub mod validator;

pub use error::{LinterError, Result, ValidationError};
pub use lnix_domain::{
    ValidationResult, classify_nix_eval_error, format_validation_result,
    format_validation_result_verbose,
};
pub use nix_eval::{NixEvalResult, eval_package, eval_package_for_arch};
pub use validator::validate_packages;
