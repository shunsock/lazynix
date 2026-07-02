//! `lnix init` — scaffold a new project from templates.

use std::fs;
use std::path::Path;

use lnix_flake_generator::LazyNixParser;

use crate::error::{LazyNixError, Result};

// TODO: consume FsProjectScaffolder instead once the init use-case
// migrates (#30); until then reach into the infra crate's templates
const YAML_TEMPLATE: &str = include_str!("../../../lnix-infra/templates/lazynix.yaml.template");
const FLAKE_TEMPLATE: &str = include_str!("../../../lnix-infra/templates/flake.nix.init.template");

/// Writes the template `lazynix.yaml` and `flake.nix`.
///
/// Without `force`, refuses to overwrite either existing file.
pub fn execute(config_dir: &Path, force: bool) -> Result<()> {
    let parser = LazyNixParser::new(config_dir.to_path_buf());
    let yaml_path = parser.config_path();
    let flake_path = Path::new("flake.nix"); // Always current directory

    if !config_dir.exists() {
        return Err(LazyNixError::Validation(format!(
            "Config directory does not exist: {}. Please create it first.",
            config_dir.display()
        )));
    }

    if !force {
        if yaml_path.exists() {
            return Err(LazyNixError::FileExists(yaml_path.display().to_string()));
        }
        if flake_path.exists() {
            return Err(LazyNixError::FileExists(flake_path.display().to_string()));
        }
    }

    parser.write_config(YAML_TEMPLATE)?;
    fs::write(flake_path, FLAKE_TEMPLATE)?;

    print_next_steps(&yaml_path);
    Ok(())
}

fn print_next_steps(yaml_path: &Path) {
    println!("✓ Initialized LazyNix project");
    println!("  - Created: {}", yaml_path.display());
    println!("  - Created: flake.nix");
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
}
