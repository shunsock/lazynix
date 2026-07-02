//! Gateway ports for external processes.
//!
//! Two execution shapes exist and are kept as separate traits because
//! their semantics differ: [`NixRunner`] inherits stdio and returns
//! only exit codes (interactive), while [`NixEvaluator`] and
//! [`VersionResolver`] capture output for the domain to interpret.

mod nix_evaluator;
mod nix_runner;
mod version_resolver;

pub use nix_evaluator::{EvalOutcome, NixEvaluator};
pub use nix_runner::NixRunner;
pub use version_resolver::{ResolvedVersion, VersionResolver};
