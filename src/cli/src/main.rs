mod cli_parser;
mod commands;
mod env_validator;
mod error;
mod lazynix_settings_yaml;
mod task_interpolator;

use clap::Parser;
use cli_parser::{Cli, Commands};
use error::Result;
use lazynix_settings_yaml::Settings;
use lnix_flake_generator::{LazyNixParser, render_flake, validate_config};
use lnix_nix_dispatcher::{
    run_flake_update, run_nix_develop, run_nix_develop_command, run_nix_test, run_task_in_nix_env,
};
use std::fs;
use std::path::Path;

const YAML_TEMPLATE: &str = include_str!("../../templates/lazynix.yaml.template");
const FLAKE_TEMPLATE: &str = include_str!("../../templates/flake.nix.init.template");

/// Read lazynix-settings.yaml from config directory
fn read_local_settings(config_dir: &Path) -> Result<Option<Settings>> {
    let settings_path = config_dir.join("lazynix-settings.yaml");

    if !settings_path.exists() {
        return Ok(None);
    }

    let settings_str = fs::read_to_string(settings_path)?;
    let settings: Settings = serde_yaml::from_str(&settings_str)?;

    if let Some(url) = &settings.override_stable_package {
        lazynix_settings_yaml::validate_registry_url(url)?;
    }

    Ok(Some(settings))
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let config_dir = &cli.config_dir; // Extract once

    match cli.command {
        Commands::Init { force } => cmd_init(config_dir, force)?,
        Commands::Test { update } => cmd_test(config_dir, update)?,
        Commands::Update => cmd_update()?,
        Commands::Develop { update } => cmd_develop(config_dir, update)?,
        Commands::Run {
            update,
            no_regen,
            command,
        } => {
            let exit_code = cmd_run(config_dir, update, !no_regen, command)?;
            if exit_code != 0 {
                std::process::exit(exit_code);
            }
        }
        Commands::Task { task_name, args } => {
            let exit_code = cmd_task(config_dir, task_name, args)?;
            if exit_code != 0 {
                std::process::exit(exit_code);
            }
        }
        Commands::Lint { verbose, arch } => {
            let config_dir_str = config_dir.to_str().unwrap_or(".");
            let success = commands::lint::execute(config_dir_str, verbose, arch.as_deref())?;
            if !success {
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

fn cmd_init(config_dir: &Path, force: bool) -> Result<()> {
    let parser = LazyNixParser::new(config_dir.to_path_buf());
    let yaml_path = parser.config_path();
    let flake_path = Path::new("flake.nix"); // Always current directory

    // Validate config directory exists
    if !config_dir.exists() {
        return Err(error::LazyNixError::Validation(format!(
            "Config directory does not exist: {}. Please create it first.",
            config_dir.display()
        )));
    }

    // Check if files exist
    if !force {
        if yaml_path.exists() {
            return Err(error::LazyNixError::FileExists(
                yaml_path.display().to_string(),
            ));
        }
        if flake_path.exists() {
            return Err(error::LazyNixError::FileExists(
                flake_path.display().to_string(),
            ));
        }
    }

    // Write template files
    parser.write_config(YAML_TEMPLATE)?;
    fs::write(flake_path, FLAKE_TEMPLATE)?;

    println!("✓ Initialized LazyNix project");
    println!("  - Created: {}", yaml_path.display());
    println!("  - Created: {}", flake_path.display());
    println!();
    println!("Next steps:");
    println!(
        "  1. Run 'git add flake.nix {}' to stage the files",
        yaml_path.display()
    );
    println!(
        "  2. Edit {} to configure your environment",
        yaml_path.display()
    );
    println!("  3. Run 'lnix develop' to generate flake.nix and enter the shell");

    Ok(())
}

fn cmd_develop(config_dir: &Path, update_lock: bool) -> Result<()> {
    // Step 1: Read settings (optional)
    let settings = read_local_settings(config_dir)?;
    let override_url = settings
        .as_ref()
        .and_then(|s| s.override_stable_package.as_deref());

    // Step 2: Read and parse config using LazyNixParser
    println!("Reading configuration from {}...", config_dir.display());
    let parser = LazyNixParser::new(config_dir.to_path_buf());
    let config = parser.read_config()?;

    // Step 3: Validate config
    println!("Validating configuration...");
    validate_config(&config)?;

    // Step 3.5: Validate env configuration
    env_validator::validate_env_config(&config.dev_shell.env, config_dir)?;

    // Step 4: Generate flake.nix
    println!("Generating flake.nix...");
    let flake_content = render_flake(&config, override_url);
    fs::write("flake.nix", flake_content)?;
    println!("✓ flake.nix generated successfully");

    // Step 4: Update flake.lock if requested
    if update_lock {
        run_flake_update()?;
    } else {
        println!("Skipping flake.lock update (use --update to update)");
    }

    // Step 5: Enter nix develop
    println!();
    run_nix_develop()?;

    Ok(())
}

fn cmd_test(config_dir: &Path, update_lock: bool) -> Result<()> {
    // Step 1: Read settings (optional)
    let settings = read_local_settings(config_dir)?;
    let override_url = settings
        .as_ref()
        .and_then(|s| s.override_stable_package.as_deref());

    // Step 2: Read and parse config using LazyNixParser
    println!("Reading configuration from {}...", config_dir.display());
    let parser = LazyNixParser::new(config_dir.to_path_buf());
    let config = parser.read_config()?;

    // Step 3: Validate config
    println!("Validating configuration...");
    validate_config(&config)?;

    // Step 3.5: Validate env configuration
    env_validator::validate_env_config(&config.dev_shell.env, config_dir)?;

    // Step 4: Validate test attribute is not empty
    if config.dev_shell.test.is_empty() {
        return Err(error::LazyNixError::Validation(
            "No test commands defined in lazynix.yaml. Add test attribute to devShell.".to_string(),
        ));
    }

    // Step 5: Generate flake.nix
    println!("Generating flake.nix...");
    let flake_content = render_flake(&config, override_url);
    fs::write("flake.nix", flake_content)?;
    println!("✓ flake.nix generated successfully");

    // Step 6: Update flake.lock if requested
    if update_lock {
        run_flake_update()?;
    } else {
        println!("Skipping flake.lock update (use --update to update)");
    }

    // Step 7: Run tests in nix develop environment
    println!();
    let exit_code = run_nix_test()?;
    if exit_code != 0 {
        std::process::exit(exit_code);
    }

    Ok(())
}

fn cmd_update() -> Result<()> {
    run_flake_update()?;
    Ok(())
}

fn cmd_run(
    config_dir: &Path,
    update_lock: bool,
    regenerate: bool,
    cmd_args: Vec<String>,
) -> Result<i32> {
    // Validate non-empty command
    if cmd_args.is_empty() {
        return Err(error::LazyNixError::Validation(
            "command arguments cannot be empty".to_string(),
        ));
    }

    if regenerate {
        // Step 1: Read settings (optional)
        let settings = read_local_settings(config_dir)?;
        let override_url = settings
            .as_ref()
            .and_then(|s| s.override_stable_package.as_deref());

        // Step 2: Read and parse config using LazyNixParser
        println!("Reading configuration from {}...", config_dir.display());
        let parser = LazyNixParser::new(config_dir.to_path_buf());
        let config = parser.read_config()?;

        // Step 3: Validate config
        println!("Validating configuration...");
        validate_config(&config)?;

        // Step 3.5: Validate env configuration
        env_validator::validate_env_config(&config.dev_shell.env, config_dir)?;

        // Step 4: Generate flake.nix
        println!("Generating flake.nix...");
        let flake_content = render_flake(&config, override_url);
        fs::write("flake.nix", flake_content)?;
        println!("✓ flake.nix generated successfully");
    } else {
        println!("Using existing flake.nix (--no-regen specified)");
    }

    // Step 5: Update flake.lock if requested
    if update_lock {
        run_flake_update()?;
    } else {
        println!("Skipping flake.lock update (use --update to update)");
    }

    // Step 6: Run the command in nix develop environment
    println!();
    println!("Running command: {}", cmd_args.join(" "));
    let exit_code = run_nix_develop_command(cmd_args)?;
    Ok(exit_code)
}

fn cmd_task(config_dir: &Path, task_name: String, args: Vec<String>) -> Result<i32> {
    // Step 1: Read and parse config using LazyNixParser
    println!("Reading configuration from {}...", config_dir.display());
    let parser = LazyNixParser::new(config_dir.to_path_buf());
    let config = parser.read_config()?;

    // Step 2: Validate config
    println!("Validating configuration...");
    validate_config(&config)?;

    // Step 3: Get task definitions
    let tasks = config.dev_shell.task.ok_or_else(|| {
        error::LazyNixError::Validation("No tasks defined in lazynix.yaml".to_string())
    })?;

    // Step 4: Find the specified task
    let task_def = tasks.get(&task_name).ok_or_else(|| {
        error::LazyNixError::Validation(format!("Task '{}' not found in lazynix.yaml", task_name))
    })?;

    // Step 5: Interpolate CLI arguments
    let commands = task_interpolator::interpolate_command(&task_def.commands, &args);

    // Step 6: Display task information
    println!("Running task: {}", task_name);
    if let Some(description) = &task_def.description {
        println!("Description: {}", description);
    }
    println!();

    // Step 7: Execute task in nix develop environment
    let exit_code = run_task_in_nix_env(commands)?;
    Ok(exit_code)
}
