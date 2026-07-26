//! Use-cases: one module per subcommand.
//!
//! Every use-case receives [`crate::Deps`] and returns
//! `Result<i32, crate::ApplicationError>` — the exit code on the happy
//! rail, a categorized failure otherwise.

mod develop;
mod generate;
mod init;
mod lint;
mod run;
mod search;
mod task;
mod test;
mod update;

pub use develop::develop;
pub use generate::generate;
pub use init::init;
pub use lint::lint;
pub use run::run;
pub use search::search;
pub use task::task;
pub use test::test;
pub use update::update;
