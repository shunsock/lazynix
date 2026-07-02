//! Application layer of LazyNix: use-cases and their wiring surface.
//!
//! This crate depends only on `lnix-domain`. It defines:
//!
//! - [`Deps`]: the borrowed bundle of ports every use-case receives.
//!   The composition root (the `lnix` binary) constructs it once from
//!   concrete adapters; tests construct it from mocks.
//! - [`ApplicationError`]: the top of the two-tier error hierarchy,
//!   lifting the domain's focused errors via `#[from]` so use-case
//!   bodies stay on the railway (`?` short-circuits every failure).
//!
//! Use-cases live under [`usecase`], one per subcommand, all shaped
//! `fn(&Deps, ...) -> Result<i32, ApplicationError>`.

mod deps;
mod error;
#[cfg(test)]
mod mocks;
mod pipeline;
mod usecase;

pub use deps::Deps;
pub use error::ApplicationError;
pub use usecase::{develop, init, lint, run, search, task, test, update};
