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
    use super::*;
    use lnix_domain::interface::gateway::{EvalOutcome, ResolvedVersion};
    use lnix_domain::{Config, ConfigError, FlakeError, NixError, PackageName, PackageVersion};

    // Building Deps from stub impls is itself the assertion that every
    // port trait stays object-safe (a generic method would break `dyn`).
    struct StubRepo;
    impl ConfigRepository for StubRepo {
        fn read_config(&self) -> Result<Config, ConfigError> {
            Ok(serde_stub_config())
        }
        fn write_config(&self, _config: &Config) -> Result<(), ConfigError> {
            Ok(())
        }
        fn read_settings(&self) -> Result<Option<lnix_domain::Settings>, ConfigError> {
            Ok(None)
        }
    }

    struct StubWriter;
    impl FlakeWriter for StubWriter {
        fn write_flake(&self, _contents: &str) -> Result<(), FlakeError> {
            Ok(())
        }
    }

    struct StubEnv;
    impl EnvFilePresenceChecker for StubEnv {
        fn exists(&self, _path: &str) -> bool {
            true
        }
    }

    struct StubNix;
    impl NixRunner for StubNix {
        fn develop(&self) -> Result<(), NixError> {
            Ok(())
        }
        fn develop_command(&self, _args: &[String]) -> Result<i32, NixError> {
            Ok(0)
        }
        fn test(&self) -> Result<i32, NixError> {
            Ok(0)
        }
        fn run_task(&self, _commands: &[String]) -> Result<i32, NixError> {
            Ok(0)
        }
        fn flake_update(&self) -> Result<(), NixError> {
            Ok(())
        }
    }

    struct StubEval;
    impl NixEvaluator for StubEval {
        fn eval_package(
            &self,
            _package: &PackageName,
            _arch: Option<&str>,
        ) -> Result<EvalOutcome, NixError> {
            Ok(EvalOutcome {
                success: true,
                stderr: String::new(),
            })
        }
    }

    struct StubResolver;
    impl VersionResolver for StubResolver {
        fn resolve(
            &self,
            _name: &PackageName,
            _version: &PackageVersion,
        ) -> Result<ResolvedVersion, NixError> {
            Ok(ResolvedVersion {
                commit: "e607cb5".to_string(),
                attr: "go_1_21".to_string(),
            })
        }
        fn search(
            &self,
            _name: &PackageName,
            _version_constraint: Option<&str>,
            _json: bool,
            _one: bool,
        ) -> Result<String, NixError> {
            Ok(String::new())
        }
    }

    struct NullOut;
    impl OutputPort for NullOut {
        fn info(&self, _message: &str) {}
        fn warn(&self, _message: &str) {}
    }

    fn serde_stub_config() -> Config {
        let yaml = "devShell:\n  package:\n    stable:\n      - name: bash\n";
        serde_yaml::from_str(yaml).unwrap()
    }

    #[test]
    fn bundles_all_ports_and_dispatches_through_dyn() {
        // Arrange
        let repo = StubRepo;
        let writer = StubWriter;
        let env = StubEnv;
        let nix = StubNix;
        let nix_eval = StubEval;
        let resolver = StubResolver;
        let out = NullOut;

        let deps = Deps {
            repo: &repo,
            writer: &writer,
            env: &env,
            nix: &nix,
            nix_eval: &nix_eval,
            resolver: &resolver,
            out: &out,
        };

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
