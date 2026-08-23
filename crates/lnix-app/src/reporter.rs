//! Port for the presentation adapter.
//!
//! Use-cases emit [`UseCaseEvent`]s through this port instead of
//! assembling text themselves. Adapters (terminal / JSON / TUI / test)
//! decide how to render each event.

use crate::event::UseCaseEvent;

pub trait ReporterPort {
    fn report(&self, event: &UseCaseEvent);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mocks::RecordingReporter;

    #[test]
    fn default_reporter_has_no_events() {
        let reporter = RecordingReporter::default();

        assert!(reporter.events().is_empty());
    }

    #[test]
    fn records_single_event() {
        let reporter = RecordingReporter::default();

        reporter.report(&UseCaseEvent::ReadingConfig);

        assert_eq!(reporter.events(), vec![UseCaseEvent::ReadingConfig]);
    }

    #[test]
    fn preserves_event_payload() {
        let reporter = RecordingReporter::default();
        let event = UseCaseEvent::RunningCommand {
            argv: vec!["echo".to_string(), "hi".to_string()],
        };

        reporter.report(&event);

        assert_eq!(reporter.events(), vec![event]);
    }

    #[test]
    fn records_events_in_order() {
        let reporter = RecordingReporter::default();

        reporter.report(&UseCaseEvent::ReadingConfig);
        reporter.report(&UseCaseEvent::ValidatingConfig);
        reporter.report(&UseCaseEvent::GeneratingFlake);

        assert_eq!(
            reporter.events(),
            vec![
                UseCaseEvent::ReadingConfig,
                UseCaseEvent::ValidatingConfig,
                UseCaseEvent::GeneratingFlake,
            ]
        );
    }
}
