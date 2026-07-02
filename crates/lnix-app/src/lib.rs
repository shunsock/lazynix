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
//! Use-cases land per subcommand (#29, #30); this crate deliberately
//! ships the skeleton first so app and infra (#28) can build against
//! the same surface in parallel.

mod deps;
mod error;

pub use deps::Deps;
pub use error::ApplicationError;
