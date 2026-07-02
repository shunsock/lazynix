//! `lnix test` — generate the flake and run declared test commands.

use crate::deps::Deps;
use crate::error::ApplicationError;
use crate::pipeline;

/// Renders `flake.nix` and runs the test commands in `nix develop`.
///
/// Fails fast (before writing the flake) when no tests are declared.
pub fn test(d: &Deps, update_lock: bool) -> Result<i32, ApplicationError> {
    let loaded = pipeline::load_config(d)?;

    if loaded.config.dev_shell.test.is_empty() {
        return Err(ApplicationError::NoTestCommands);
    }

    pipeline::write_flake(d, &loaded)?;
    pipeline::maybe_update_lock(d, update_lock)?;

    d.out.info("");
    Ok(d.nix.test()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mocks::*;

    #[test]
    fn runs_declared_tests_after_writing_flake() {
        // Arrange
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable:\n      - name: bash\n  test:\n    - cargo test\n",
        ));

        // Act
        let code = test(&m.deps(), false).unwrap();

        // Assert
        assert_eq!(code, 0);
        assert!(m.writer.written().is_some());
        assert_eq!(m.nix.test_calls(), 1);
    }

    #[test]
    fn fails_fast_when_no_tests_declared() {
        // Arrange
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable:\n      - name: bash\n",
        ));

        // Act
        let result = test(&m.deps(), false);

        // Assert
        assert!(matches!(result, Err(ApplicationError::NoTestCommands)));
        assert!(m.writer.written().is_none());
        assert_eq!(m.nix.test_calls(), 0);
    }
}
