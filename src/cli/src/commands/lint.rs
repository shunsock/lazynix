//! Lint command implementation

use crate::error::Result;
use lnix_flake_generator::LazyNixParser;
use lnix_linter::{format_validation_result, format_validation_result_verbose, validate_packages};
use std::path::PathBuf;

/// Execute the lint command
///
/// # Arguments
/// * `config_dir` - Directory containing lazynix.yaml
/// * `verbose` - Show verbose error details
/// * `arch` - Override target architecture
///
/// # Returns
/// Ok(true) if all packages are valid, Ok(false) if validation errors exist
pub fn execute(config_dir: &str, verbose: bool, arch: Option<&str>) -> Result<bool> {
    // Parse lazynix.yaml
    let parser = LazyNixParser::new(PathBuf::from(config_dir));
    let config = parser.read_config()?;

    // Extract all package names (stable + unstable)
    let mut packages: Vec<String> = Vec::new();

    // Add stable packages
    packages.extend(config.dev_shell.package.stable.iter().cloned());

    // Add unstable packages
    packages.extend(config.dev_shell.package.unstable.iter().cloned());

    if packages.is_empty() {
        println!("No packages to validate.");
        return Ok(true);
    }

    // Validate packages
    let result = validate_packages(&packages, arch);

    // Format and print results
    let output = if verbose {
        format_validation_result_verbose(&result)
    } else {
        format_validation_result(&result)
    };

    print!("{}", output);

    // Return success status
    Ok(result.errors.is_empty())
}
