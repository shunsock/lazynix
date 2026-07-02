//! Subprocess adapters for the domain's gateway ports.

mod nix_evaluator;
mod nix_runner;
mod version_resolver;

pub use nix_evaluator::SubprocessNixEvaluator;
pub use nix_runner::SubprocessNixRunner;
pub use version_resolver::NixVersionsResolver;
