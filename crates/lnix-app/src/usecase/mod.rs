//! Use-cases: one module per subcommand.
//!
//! Every use-case receives [`crate::Deps`] and returns
//! `Result<i32, crate::ApplicationError>` — the exit code on the happy
//! rail, a categorized failure otherwise.

mod develop;

pub use develop::develop;
