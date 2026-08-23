//! `lnix update` — update `flake.lock` without entering the shell.

use crate::deps::Deps;
use crate::error::ApplicationError;
use crate::pipeline;

/// Runs `nix flake update`.
pub fn update(d: &Deps) -> Result<i32, ApplicationError> {
    pipeline::maybe_update_lock(d, true)?;
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::UseCaseEvent;
    use crate::mocks::*;

    #[test]
    fn updates_the_lock_unconditionally() {
        let m = Mocks::with_missing_config();

        let code = update(&m.deps()).unwrap();

        assert_eq!(code, 0);
        assert_eq!(m.nix.flake_update_calls(), 1);
        assert!(
            m.reporter
                .events()
                .iter()
                .any(|e| matches!(e, UseCaseEvent::LockUpdated))
        );
    }
}
