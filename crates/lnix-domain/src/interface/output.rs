//! Port for the user-facing display sink.

/// Sink for user-facing progress messages and warnings.
///
/// Message *assembly* (report formatting, diagnostics) is pure domain
/// logic; this port only carries the finished strings to the user.
pub trait OutputPort {
    /// Emits a progress or result message.
    fn info(&self, message: &str);

    /// Emits a non-fatal warning.
    fn warn(&self, message: &str);
}
