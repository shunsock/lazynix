//! The dependency bundle every use-case receives.
//!
//! One context struct (rather than per-use-case structs or generic
//! type parameters) keeps use-case signatures flat as ports are added,
//! and lets the composition root wire adapters exactly once. `&dyn`
//! dispatch is negligible for a CLI, and swapping mocks in tests is a
//! plain struct literal.

use lnix_domain::interface::gateway::{NixEvaluator, NixRunner, VersionResolver};
use lnix_domain::interface::output::OutputPort;
use lnix_domain::interface::persistence::{ConfigRepository, EnvFilePresenceChecker, FlakeWriter};

/// Borrowed bundle of every port a use-case may touch.
///
/// The composition root (the binary) owns the adapter values; `Deps`
/// only borrows them for the duration of one command.
pub struct Deps<'a> {
    /// Reads/writes `lazynix.yaml` and reads `lazynix-settings.yaml`.
    pub repo: &'a dyn ConfigRepository,
    /// Persists rendered `flake.nix` content.
    pub writer: &'a dyn FlakeWriter,
    /// Checks dotenv files referenced by the config exist.
    pub env: &'a dyn EnvFilePresenceChecker,
    /// Runs interactive `nix` commands (develop/test/task/update).
    pub nix: &'a dyn NixRunner,
    /// Evaluates package availability via `nix eval` (capturing).
    pub nix_eval: &'a dyn NixEvaluator,
    /// Resolves/searches package versions via nix-versions (capturing).
    pub resolver: &'a dyn VersionResolver,
    /// Sink for user-facing progress messages and warnings.
    pub out: &'a dyn OutputPort,
}

#[cfg(test)]
mod tests {
    use crate::mocks::*;

    // Building Deps from the shared mocks is itself the assertion that
    // every port trait stays object-safe (a generic method would break
    // `dyn`).
    #[test]
    fn bundles_all_ports_and_dispatches_through_dyn() {
        // Arrange
        let m = Mocks::with_config(config_from_yaml(
            "devShell:\n  package:\n    stable:\n      - name: bash\n",
        ));
        let deps = m.deps();

        // Act
        let config = deps.repo.read_config().unwrap();
        let write_result = deps.writer.write_flake("{}");
        let env_exists = deps.env.exists(".env");
        let exit_code = deps.nix.develop_command(&["true".to_string()]).unwrap();
        let outcome = deps
            .nix_eval
            .eval_package(&"bash".parse().unwrap(), None)
            .unwrap();
        let resolved = deps
            .resolver
            .resolve(&"go".parse().unwrap(), &"1.21.13".parse().unwrap())
            .unwrap();
        deps.out.info("progress");

        // Assert
        assert_eq!(config.dev_shell.package.stable[0].name.as_str(), "bash");
        assert!(write_result.is_ok());
        assert!(env_exists);
        assert_eq!(exit_code, 0);
        assert!(outcome.success);
        assert_eq!(resolved.attr, "go_1_21");
    }
}
