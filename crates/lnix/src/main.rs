mod cli_parser;
mod commands;
mod composition;
mod env_validator;
mod error;

use std::process;

use clap::Parser;

use cli_parser::{Cli, Commands};
use composition::AdapterSet;
use error::Result;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let config_dir = &cli.config_dir;

    match cli.command {
        Commands::Init { force } => commands::init::execute(config_dir, force)?,
        Commands::Update => commands::update::execute()?,
        Commands::Develop { update } => {
            let adapters = AdapterSet::new(config_dir);
            exit_on_failure(run_usecase(lnix_app::develop(&adapters.deps(), update)));
        }
        Commands::Test { update } => commands::test::execute(config_dir, update)?,
        Commands::Run {
            update,
            no_regen,
            command,
        } => exit_on_failure(commands::run::execute(
            config_dir, update, !no_regen, command,
        )?),
        Commands::Task { task_name, args } => {
            exit_on_failure(commands::task::execute(config_dir, task_name, args)?)
        }
        Commands::Lint { verbose, arch } => {
            let config_dir_str = config_dir.to_str().unwrap_or(".");
            let success = commands::lint::execute(config_dir_str, verbose, arch.as_deref())?;
            if !success {
                process::exit(1);
            }
        }
        Commands::Search {
            package_name,
            version,
            json,
            one,
        } => commands::search::execute(&package_name, version.as_deref(), json, one)?,
    }

    Ok(())
}

/// Propagates a subprocess exit code as this process's exit code.
fn exit_on_failure(exit_code: i32) {
    if exit_code != 0 {
        process::exit(exit_code);
    }
}

/// Unwraps a use-case result: failures print like every other command
/// error ("Error: ...") and exit 1; successes yield the exit code.
fn run_usecase(result: std::result::Result<i32, lnix_app::ApplicationError>) -> i32 {
    match result {
        Ok(code) => code,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}
