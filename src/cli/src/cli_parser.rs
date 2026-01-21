use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "lnix")]
#[command(about = "LazyNix - YAML-to-Nix transpiler for lazy engineers", long_about = None)]
#[command(version)]
pub struct Cli {
    /// Directory containing lazynix.yaml and lazynix-settings.yaml
    ///
    /// Can also be set via LAZYNIX_CONFIG_DIR environment variable.
    /// Defaults to current directory if neither is specified.
    #[arg(
        short = 'C',
        long = "config-dir",
        global = true,
        env = "LAZYNIX_CONFIG_DIR",
        default_value = ".",
        value_name = "DIR"
    )]
    pub config_dir: PathBuf,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize a new LazyNix project with template files
    Init {
        /// Overwrite existing files without prompting
        #[arg(short, long)]
        force: bool,
    },

    /// Update flake.lock without entering development shell
    Update,

    /// Generate flake.nix from lazynix.yaml and enter nix develop shell
    Develop {
        /// Update flake.lock before entering the shell
        #[arg(long)]
        update: bool,
    },

    /// Run a command in the development environment
    Run {
        /// Update flake.lock before running the command
        #[arg(long)]
        update: bool,

        /// Skip regenerating flake.nix from lazynix.yaml
        #[arg(long)]
        no_regen: bool,

        /// The command to run in the development environment
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        command: Vec<String>,
    },

    /// Run tests defined in lazynix.yaml
    Test {
        /// Update flake.lock before running tests
        #[arg(long)]
        update: bool,
    },

    /// Run a task defined in lazynix.yaml
    Task {
        /// Name of the task to run
        task_name: String,

        /// Arguments to pass to the task
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Validate packages in lazynix.yaml
    Lint {
        /// Show verbose error details (raw nix eval output)
        #[arg(short, long)]
        verbose: bool,

        /// Override target architecture (e.g., aarch64-darwin, x86_64-linux)
        #[arg(long)]
        arch: Option<String>,
    },
}
