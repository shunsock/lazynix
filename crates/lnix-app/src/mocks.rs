//! Recording mocks shared by every use-case test.
//!
//! `Mocks` owns one mock per port and lends them out as a [`Deps`], so
//! a test is: build `Mocks`, run the use-case, assert on recordings.
//! No filesystem, no subprocess, no terminal.

use std::cell::RefCell;

use lnix_domain::interface::gateway::{
    EvalOutcome, NixEvaluator, NixRunner, ResolvedVersion, VersionResolver,
};
use lnix_domain::interface::output::OutputPort;
use lnix_domain::interface::persistence::{
    ConfigRepository, EnvFilePresenceChecker, FlakeWriter, ProjectScaffolder,
};
use lnix_domain::{
    ConfigError, DevShellDefinition, FlakeError, NixError, PackageName, PackageVersion, Settings,
};

use crate::deps::Deps;

pub(crate) fn config_from_yaml(yaml: &str) -> DevShellDefinition {
    serde_yaml::from_str(yaml).unwrap()
}

pub(crate) struct MockRepo {
    config: Option<DevShellDefinition>,
    persisted: RefCell<Option<DevShellDefinition>>,
}

impl ConfigRepository for MockRepo {
    fn read_config(&self) -> Result<DevShellDefinition, ConfigError> {
        self.config
            .clone()
            .ok_or_else(|| ConfigError::NotFound(".".to_string()))
    }

    fn write_config(&self, config: &DevShellDefinition) -> Result<(), ConfigError> {
        *self.persisted.borrow_mut() = Some(config.clone());
        Ok(())
    }

    fn read_settings(&self) -> Result<Option<Settings>, ConfigError> {
        Ok(None)
    }
}

impl MockRepo {
    pub(crate) fn persisted_config(&self) -> Option<DevShellDefinition> {
        self.persisted.borrow().clone()
    }
}

#[derive(Default)]
pub(crate) struct SpyWriter {
    written: RefCell<Option<String>>,
}

impl FlakeWriter for SpyWriter {
    fn write_flake(&self, contents: &str) -> Result<(), FlakeError> {
        *self.written.borrow_mut() = Some(contents.to_string());
        Ok(())
    }
}

impl SpyWriter {
    pub(crate) fn written(&self) -> Option<String> {
        self.written.borrow().clone()
    }
}

pub(crate) struct StubEnvChecker {
    all_present: bool,
}

impl EnvFilePresenceChecker for StubEnvChecker {
    fn exists(&self, _path: &str) -> bool {
        self.all_present
    }
}

#[derive(Default)]
pub(crate) struct MockScaffolder {
    pub(crate) config_present: bool,
    pub(crate) flake_present: bool,
    config_written: RefCell<bool>,
    flake_written: RefCell<bool>,
}

impl ProjectScaffolder for MockScaffolder {
    fn config_exists(&self) -> bool {
        self.config_present
    }

    fn flake_exists(&self) -> bool {
        self.flake_present
    }

    fn config_path_display(&self) -> String {
        "./lazynix.yaml".to_string()
    }

    fn flake_path_display(&self) -> String {
        "./flake.nix".to_string()
    }

    fn write_config_template(&self) -> Result<(), ConfigError> {
        *self.config_written.borrow_mut() = true;
        Ok(())
    }

    fn write_flake_template(&self) -> Result<(), FlakeError> {
        *self.flake_written.borrow_mut() = true;
        Ok(())
    }
}

impl MockScaffolder {
    pub(crate) fn config_written(&self) -> bool {
        *self.config_written.borrow()
    }

    pub(crate) fn flake_written(&self) -> bool {
        *self.flake_written.borrow()
    }
}

#[derive(Default)]
pub(crate) struct FakeNix {
    develop_calls: RefCell<u32>,
    flake_update_calls: RefCell<u32>,
    test_calls: RefCell<u32>,
    run_task_commands: RefCell<Option<Vec<String>>>,
    develop_command_args: RefCell<Option<Vec<String>>>,
}

impl NixRunner for FakeNix {
    fn develop(&self) -> Result<(), NixError> {
        *self.develop_calls.borrow_mut() += 1;
        Ok(())
    }

    fn develop_command(&self, args: &[String]) -> Result<i32, NixError> {
        *self.develop_command_args.borrow_mut() = Some(args.to_vec());
        Ok(0)
    }

    fn test(&self) -> Result<i32, NixError> {
        *self.test_calls.borrow_mut() += 1;
        Ok(0)
    }

    fn run_task(&self, commands: &[String]) -> Result<i32, NixError> {
        *self.run_task_commands.borrow_mut() = Some(commands.to_vec());
        Ok(0)
    }

    fn flake_update(&self) -> Result<(), NixError> {
        *self.flake_update_calls.borrow_mut() += 1;
        Ok(())
    }
}

impl FakeNix {
    pub(crate) fn develop_calls(&self) -> u32 {
        *self.develop_calls.borrow()
    }

    pub(crate) fn flake_update_calls(&self) -> u32 {
        *self.flake_update_calls.borrow()
    }

    pub(crate) fn test_calls(&self) -> u32 {
        *self.test_calls.borrow()
    }

    pub(crate) fn run_task_commands(&self) -> Option<Vec<String>> {
        self.run_task_commands.borrow().clone()
    }

    pub(crate) fn develop_command_args(&self) -> Option<Vec<String>> {
        self.develop_command_args.borrow().clone()
    }
}

#[derive(Default)]
pub(crate) struct StubEvaluator {
    failing: Vec<String>,
}

impl NixEvaluator for StubEvaluator {
    fn eval_package(
        &self,
        package: &PackageName,
        _arch: Option<&str>,
    ) -> Result<EvalOutcome, NixError> {
        if self.failing.iter().any(|name| name == package.as_str()) {
            return Ok(EvalOutcome {
                success: false,
                stderr: format!(
                    "error: attribute '{}' does not provide attribute 'outPath'",
                    package
                ),
            });
        }
        Ok(EvalOutcome {
            success: true,
            stderr: String::new(),
        })
    }
}

#[derive(Default)]
pub(crate) struct StubResolver {
    failing: Vec<(String, String)>,
    resolve_calls: RefCell<Vec<String>>,
    infra_failure: bool,
}

impl VersionResolver for StubResolver {
    fn resolve(
        &self,
        name: &PackageName,
        version: &PackageVersion,
    ) -> Result<ResolvedVersion, NixError> {
        self.resolve_calls.borrow_mut().push(name.to_string());
        if self.infra_failure {
            return Err(NixError::NoExitCode);
        }
        if let Some((_, message)) = self.failing.iter().find(|(n, _)| n == name.as_str()) {
            return Err(NixError::VersionResolution {
                spec: format!("{}@{}", name, version),
                message: message.clone(),
            });
        }
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
        Ok("go 1.21.13 nixpkgs/e607cb5#go_1_21".to_string())
    }
}

impl StubResolver {
    pub(crate) fn resolve_calls(&self) -> Vec<String> {
        self.resolve_calls.borrow().clone()
    }
}

#[derive(Default)]
pub(crate) struct RecordingOutput {
    infos: RefCell<Vec<String>>,
    warns: RefCell<Vec<String>>,
}

impl OutputPort for RecordingOutput {
    fn info(&self, message: &str) {
        self.infos.borrow_mut().push(message.to_string());
    }

    fn warn(&self, message: &str) {
        self.warns.borrow_mut().push(message.to_string());
    }
}

impl RecordingOutput {
    pub(crate) fn infos(&self) -> Vec<String> {
        self.infos.borrow().clone()
    }

    pub(crate) fn warns(&self) -> Vec<String> {
        self.warns.borrow().clone()
    }
}

/// One mock per port, lent out as a [`Deps`].
pub(crate) struct Mocks {
    pub(crate) repo: MockRepo,
    pub(crate) writer: SpyWriter,
    pub(crate) env: StubEnvChecker,
    pub(crate) scaffolder: MockScaffolder,
    pub(crate) nix: FakeNix,
    pub(crate) nix_eval: StubEvaluator,
    pub(crate) resolver: StubResolver,
    pub(crate) out: RecordingOutput,
}

impl Mocks {
    pub(crate) fn with_config(config: DevShellDefinition) -> Self {
        Self::build(Some(config))
    }

    pub(crate) fn with_missing_config() -> Self {
        Self::build(None)
    }

    pub(crate) fn with_missing_env_files(mut self) -> Self {
        self.env = StubEnvChecker { all_present: false };
        self
    }

    pub(crate) fn with_existing_scaffold(mut self, config: bool, flake: bool) -> Self {
        self.scaffolder.config_present = config;
        self.scaffolder.flake_present = flake;
        self
    }

    pub(crate) fn with_failing_packages(mut self, names: &[&str]) -> Self {
        self.nix_eval.failing = names.iter().map(|name| name.to_string()).collect();
        self
    }

    pub(crate) fn with_failing_versions(mut self, entries: &[(&str, &str)]) -> Self {
        self.resolver.failing = entries
            .iter()
            .map(|(name, message)| (name.to_string(), message.to_string()))
            .collect();
        self
    }

    pub(crate) fn with_resolver_infra_failure(mut self) -> Self {
        self.resolver.infra_failure = true;
        self
    }

    fn build(config: Option<DevShellDefinition>) -> Self {
        Self {
            repo: MockRepo {
                config,
                persisted: RefCell::new(None),
            },
            writer: SpyWriter::default(),
            env: StubEnvChecker { all_present: true },
            scaffolder: MockScaffolder::default(),
            nix: FakeNix::default(),
            nix_eval: StubEvaluator::default(),
            resolver: StubResolver::default(),
            out: RecordingOutput::default(),
        }
    }

    pub(crate) fn deps(&self) -> Deps<'_> {
        Deps {
            repo: &self.repo,
            writer: &self.writer,
            env: &self.env,
            scaffolder: &self.scaffolder,
            nix: &self.nix,
            nix_eval: &self.nix_eval,
            resolver: &self.resolver,
            out: &self.out,
        }
    }
}
