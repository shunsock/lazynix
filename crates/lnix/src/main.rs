mod cli_parser;
mod composition;

use std::process;

use clap::Parser;
use lnix_app::{ApplicationError, Deps};

use cli_parser::{Cli, Commands};
use composition::AdapterSet;

fn main() {
    let cli = Cli::parse();
    let adapters = AdapterSet::new(&cli.config_dir);

    match route(cli.command, &adapters.deps()) {
        Ok(code) => {
            if code != 0 {
                process::exit(code);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

/// Dispatches the parsed subcommand into its use-case. This is the
/// only place clap types meet the application layer.
fn route(command: Commands, d: &Deps) -> Result<i32, ApplicationError> {
    match command {
        Commands::Init { force } => lnix_app::init(d, force),
        Commands::Update => lnix_app::update(d),
        Commands::Develop { update } => lnix_app::develop(d, update),
        Commands::Test { update } => lnix_app::test(d, update),
        Commands::Run {
            update,
            no_regen,
            command,
        } => lnix_app::run(d, update, !no_regen, command),
        Commands::Task { task_name, args } => lnix_app::task(d, &task_name, &args),
        Commands::Lint { verbose, arch } => lnix_app::lint(d, verbose, arch.as_deref()),
        Commands::Search {
            package_name,
            version,
            json,
            one,
        } => lnix_app::search(d, &package_name, version.as_deref(), json, one),
    }
}
