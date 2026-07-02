//! Composition root: constructs concrete adapters and lends them to
//! use-cases as [`Deps`].
//!
//! This is the only place that knows both the ports and their
//! implementations; use-cases see `&dyn` ports only.

use std::path::Path;

use lnix_app::Deps;
use lnix_infra::WorkspacePaths;
use lnix_infra::gateway::{NixVersionsResolver, SubprocessNixEvaluator, SubprocessNixRunner};
use lnix_infra::output::TerminalOutput;
use lnix_infra::persistence::{
    FsConfigRepository, FsEnvFileChecker, FsFlakeWriter, FsProjectScaffolder,
};

/// Owns one adapter per port for the duration of a command.
pub struct AdapterSet {
    repo: FsConfigRepository,
    writer: FsFlakeWriter,
    env: FsEnvFileChecker,
    scaffolder: FsProjectScaffolder,
    nix: SubprocessNixRunner,
    nix_eval: SubprocessNixEvaluator,
    resolver: NixVersionsResolver,
    out: TerminalOutput,
}

impl AdapterSet {
    pub fn new(config_dir: &Path) -> Self {
        let paths = WorkspacePaths::new(config_dir);
        Self {
            repo: FsConfigRepository::new(paths.clone()),
            writer: FsFlakeWriter::new(paths.clone()),
            env: FsEnvFileChecker::new(paths.clone()),
            scaffolder: FsProjectScaffolder::new(paths),
            nix: SubprocessNixRunner,
            nix_eval: SubprocessNixEvaluator,
            resolver: NixVersionsResolver,
            out: TerminalOutput,
        }
    }

    pub fn deps(&self) -> Deps<'_> {
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
