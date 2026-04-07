use std::process::Command;

use serde::Deserialize;

use crate::error::{NixDispatcherError, Result};

#[derive(Debug, Deserialize)]
pub struct NixVersionResult {
    pub spec: String,
    pub name: String,
    pub version: String,
    pub installable: String,
}

#[derive(Debug, Clone)]
pub struct ResolvedVersion {
    pub commit: String,
    pub attr: String,
}

fn parse_installable(installable: &str) -> Result<ResolvedVersion> {
    let rest = installable.strip_prefix("nixpkgs/").ok_or_else(|| {
        NixDispatcherError::CommandExecution(format!(
            "Unexpected installable format (missing nixpkgs/ prefix): {}",
            installable
        ))
    })?;
    let (commit, attr) = rest.split_once('#').ok_or_else(|| {
        NixDispatcherError::CommandExecution(format!(
            "Unexpected installable format (missing # separator): {}",
            installable
        ))
    })?;
    Ok(ResolvedVersion {
        commit: commit.to_string(),
        attr: attr.to_string(),
    })
}

pub fn resolve_version(package_name: &str, version: &str) -> Result<ResolvedVersion> {
    let spec = format!("{}@{}", package_name, version);
    let output = Command::new("nix")
        .arg("run")
        .arg("github:vic/nix-versions")
        .arg("--")
        .arg("--json")
        .arg("--one")
        .arg(&spec)
        .output()
        .map_err(|e| {
            NixDispatcherError::CommandExecution(format!("Failed to execute nix-versions: {}", e))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(NixDispatcherError::CommandExecution(format!(
            "nix-versions failed for '{}': {}",
            spec, stderr
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let results: Vec<NixVersionResult> = serde_json::from_str(&stdout).map_err(|e| {
        NixDispatcherError::CommandExecution(format!("Failed to parse nix-versions output: {}", e))
    })?;

    let result = results.into_iter().next().ok_or_else(|| {
        NixDispatcherError::CommandExecution(format!(
            "No version found for '{}'. Use 'lnix search {}' to find available versions.",
            spec, package_name
        ))
    })?;

    parse_installable(&result.installable)
}

/// Search for package versions via nix-versions.
///
/// Calls `nix run github:vic/nix-versions -- [flags] '<spec>'`
/// and returns the raw output.
pub fn search_versions(
    package_name: &str,
    version_constraint: Option<&str>,
    json: bool,
    one: bool,
) -> Result<String> {
    let spec = match version_constraint {
        Some(constraint) => format!("{}@{}", package_name, constraint),
        None => package_name.to_string(),
    };

    let mut cmd = Command::new("nix");
    cmd.arg("run").arg("github:vic/nix-versions").arg("--");

    if json {
        cmd.arg("--json");
    } else {
        cmd.arg("--text");
    }

    if one {
        cmd.arg("--one");
    }

    cmd.arg(&spec);

    let output = cmd.output().map_err(|e| {
        NixDispatcherError::CommandExecution(format!("Failed to execute nix-versions: {}", e))
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(NixDispatcherError::CommandExecution(format!(
            "nix-versions search failed for '{}': {}",
            spec, stderr
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_installable_valid() {
        let result = parse_installable("nixpkgs/5ed6275#go_1_21").unwrap();
        assert_eq!(result.commit, "5ed6275");
        assert_eq!(result.attr, "go_1_21");
    }

    #[test]
    fn test_parse_installable_missing_prefix() {
        assert!(parse_installable("invalid/5ed6275#go_1_21").is_err());
    }

    #[test]
    fn test_parse_installable_missing_hash() {
        assert!(parse_installable("nixpkgs/5ed6275").is_err());
    }
}
