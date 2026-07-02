//! Terminal-backed [`OutputPort`].

use lnix_domain::interface::output::OutputPort;

/// Prints progress to stdout and warnings to stderr.
///
/// The `Warning: ` prefix lives here (presentation), not in the
/// domain: diagnostics carry only their message.
pub struct TerminalOutput;

impl OutputPort for TerminalOutput {
    fn info(&self, message: &str) {
        println!("{message}");
    }

    fn warn(&self, message: &str) {
        eprintln!("Warning: {message}");
    }
}
