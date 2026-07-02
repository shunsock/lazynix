//! `lnix task` — run a named task defined in `lazynix.yaml`.

use std::path::Path;

use lnix_domain::{TaskName, interpolate_command, validate_config};
use lnix_flake_generator::LazyNixParser;
use lnix_nix_dispatcher::run_task_in_nix_env;

use crate::error::{LazyNixError, Result};

/// Looks up `task_name`, interpolates `args` into its commands, and runs
/// them sequentially in `nix develop`. Returns the task's exit code.
pub fn execute(config_dir: &Path, task_name: String, args: Vec<String>) -> Result<i32> {
    let task_name: TaskName = task_name.parse()?;

    println!("Reading configuration from {}...", config_dir.display());
    let parser = LazyNixParser::new(config_dir.to_path_buf());
    let config = parser.read_config()?;

    println!("Validating configuration...");
    // TODO: route diagnostics through OutputPort once use-cases land (#27/#29)
    for diagnostic in validate_config(&config)? {
        eprintln!("Warning: {}", diagnostic);
    }

    let tasks = config
        .dev_shell
        .task
        .ok_or_else(|| LazyNixError::Validation("No tasks defined in lazynix.yaml".to_string()))?;
    let task_def = tasks.get(&task_name).ok_or_else(|| {
        LazyNixError::Validation(format!("Task '{}' not found in lazynix.yaml", task_name))
    })?;

    let commands = interpolate_command(&task_def.commands, &args);

    println!("Running task: {}", task_name);
    if let Some(description) = &task_def.description {
        println!("Description: {}", description);
    }
    println!();

    let exit_code = run_task_in_nix_env(commands)?;
    Ok(exit_code)
}
