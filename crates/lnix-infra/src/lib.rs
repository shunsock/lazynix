//! Infrastructure layer of LazyNix: adapters for the domain ports.
//!
//! Every trait in `lnix_domain::interface` gets its concrete
//! implementation here:
//!
//! - [`persistence`] — filesystem adapters, anchored to
//!   [`WorkspacePaths`] so no adapter reads the current working
//!   directory.
//! - [`gateway`] — subprocess adapters for `nix`, funnelled through
//!   two private helpers (`run_inherit` / `run_capture`) so stdio
//!   wiring and error mapping live in one place.
//! - [`output`] — the terminal sink.
//!
//! The composition root (the `lnix` binary) constructs these and lends
//! them to use-cases via `lnix_app::Deps`.

mod paths;
mod process;

pub mod gateway;
pub mod output;
pub mod persistence;

pub use paths::WorkspacePaths;
