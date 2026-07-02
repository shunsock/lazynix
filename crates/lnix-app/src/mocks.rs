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
use lnix_domain::interface::persistence::{ConfigRepository, EnvFilePresenceChecker, FlakeWriter};
use lnix_domain::{
    Config, ConfigError, FlakeError, NixError, PackageName, PackageVersion, Settings,
};

use crate::deps::Deps;

pub(crate) fn config_from_yaml(yaml: &str) -> Config {
    serde_yaml::from_str(yaml).unwrap()
}

pub(crate) struct MockRepo {
    config: Option<Config>,
    persisted: RefCell<Option<Config>>,
}

impl ConfigRepository for MockRepo {
    fn read_config(&self) -> Result<Config, ConfigError> {
        self.config
            .clone()
            .ok_or_else(|| ConfigError::NotFound(".".to_string()))
    }

    fn write_config(&self, config: &Config) -> Result<(), ConfigError> {
        *self.persisted.borrow_mut() = Some(config.clone());
        Ok(())
    }

    fn read_settings(&self) -> Result<Option<Settings>, ConfigError> {
        Ok(None)
    }
}

impl MockRepo {
    pub(crate) fn persisted_config(&self) -> Option<Config> {
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
pub(crate) struct FakeNix {
    develop_calls: RefCell<u32>,
    flake_update_calls: RefCell<u32>,
}

impl NixRunner for FakeNix {
    fn develop(&self) -> Result<(), NixError> {
        *self.develop_calls.borrow_mut() += 1;
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
}

pub(crate) struct StubEvaluator;

impl NixEvaluator for StubEvaluator {
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

pub(crate) struct StubResolver;

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
        Ok("go 1.21.13 nixpkgs/e607cb5#go_1_21".to_string())
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
    pub(crate) nix: FakeNix,
    pub(crate) nix_eval: StubEvaluator,
    pub(crate) resolver: StubResolver,
    pub(crate) out: RecordingOutput,
}

impl Mocks {
    pub(crate) fn with_config(config: Config) -> Self {
        Self::build(Some(config))
    }

    pub(crate) fn with_missing_config() -> Self {
        Self::build(None)
    }

    pub(crate) fn with_missing_env_files(mut self) -> Self {
        self.env = StubEnvChecker { all_present: false };
        self
    }

    fn build(config: Option<Config>) -> Self {
        Self {
            repo: MockRepo {
                config,
                persisted: RefCell::new(None),
            },
            writer: SpyWriter::default(),
            env: StubEnvChecker { all_present: true },
            nix: FakeNix::default(),
            nix_eval: StubEvaluator,
            resolver: StubResolver,
            out: RecordingOutput::default(),
        }
    }

    pub(crate) fn deps(&self) -> Deps<'_> {
        Deps {
            repo: &self.repo,
            writer: &self.writer,
            env: &self.env,
            nix: &self.nix,
            nix_eval: &self.nix_eval,
            resolver: &self.resolver,
            out: &self.out,
        }
    }
}
