//! LazyNix Linter Library
//!
//! This library provides package validation functionality using `nix eval`
//! to check package existence and architecture compatibility before compilation.

#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

pub mod error;
pub mod error_classifier;
pub mod nix_eval;
pub mod reporter;
pub mod validator;

// Re-export commonly used types
pub use error::{LinterError, Result, ValidationError};
pub use error_classifier::classify_nix_eval_error;
pub use nix_eval::{NixEvalResult, eval_package, eval_package_for_arch};
pub use reporter::{format_validation_result, format_validation_result_verbose};
pub use validator::{ValidationResult, validate_packages};
