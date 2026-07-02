//! `lnix run` — run a command inside the dev environment.

use crate::deps::Deps;
use crate::error::ApplicationError;
use crate::pipeline;

/// Renders `flake.nix` (unless the caller opted out) and runs
/// `cmd_args` in `nix develop`. Returns the command's exit code.
pub fn run(
    d: &Deps,
    update_lock: bool,
    regenerate: bool,
    cmd_args: Vec<String>,
) -> Result<i32, ApplicationError> {
    if cmd_args.is_empty() {
        return Err(ApplicationError::EmptyRunCommand);
    }

    if regenerate {
        let loaded = pipeline::load_config(d)?;
        pipeline::write_flake(d, &loaded)?;
    } else {
        d.out
            .info("Using existing flake.nix (--no-regen specified)");
    }

    pipeline::maybe_update_lock(d, update_lock)?;

    d.out.info("");
    d.out
        .info(&format!("Running command: {}", cmd_args.join(" ")));
    Ok(d.nix.develop_command(&cmd_args)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mocks::*;

    #[test]
    fn regenerates_flake_and_runs_command() {
        // Arrange
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable:\n      - name: bash\n",
        ));

        // Act
        let code = run(&m.deps(), false, true, vec!["echo".into(), "hi".into()]).unwrap();

        // Assert
        assert_eq!(code, 0);
        assert!(m.writer.written().is_some());
        assert_eq!(
            m.nix.develop_command_args().unwrap(),
            vec!["echo".to_string(), "hi".to_string()]
        );
    }

    #[test]
    fn no_regen_skips_config_loading_entirely() {
        // Arrange: no config on disk — --no-regen must not need one
        let m = Mocks::with_missing_config();

        // Act
        let code = run(&m.deps(), false, false, vec!["true".into()]).unwrap();

        // Assert
        assert_eq!(code, 0);
        assert!(m.writer.written().is_none());
        assert!(
            m.out
                .infos()
                .contains(&"Using existing flake.nix (--no-regen specified)".to_string())
        );
    }

    #[test]
    fn rejects_empty_command() {
        // Arrange
        let m = Mocks::with_missing_config();

        // Act
        let result = run(&m.deps(), false, true, vec![]);

        // Assert
        assert!(matches!(result, Err(ApplicationError::EmptyRunCommand)));
    }
}
