//! Subcommand implementations.
//!
//! Each subcommand lives in its own module and shares the flake
//! generation prefix via [`pipeline`]. `main` only parses arguments and
//! dispatches into these `execute` functions.

pub mod develop;
pub mod init;
pub mod lint;
pub mod pipeline;
pub mod run;
pub mod search;
pub mod task;
pub mod test;
pub mod update;
