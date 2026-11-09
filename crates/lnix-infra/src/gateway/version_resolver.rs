//! nix-versions-backed [`VersionResolver`].

use std::process::Command;

use serde::Deserialize;

use lnix_domain::interface::gateway::{ResolvedVersion, VersionResolver};
use lnix_domain::{NixError, PackageName, PackageVersion};

use super::devbox::{self, ensure_success, version_resolution_error};
use crate::process::run_capture;

/// One entry of nix-versions' `--json` output.
#[derive(Debug, Deserialize)]
struct NixVersionEntry {
    installable: String,
}

/// nix-versions' raw split of `nixpkgs/<commit>#<attr>`.
///
/// NOTE: distinct from [`ResolvedVersion`] because `commit` here may
/// still be nix-versions' 7-digit short SHA; only after
/// [`devbox::expand_to_full_sha`] does it become the full 40-digit
/// SHA that a pinned-input `rev` requires.
struct ParsedInstallable {
    commit: String,
    attr: String,
}

/// Resolves versions by running `nix run github:vic/nix-versions`.
pub struct NixVersionsResolver;

fn nix_versions() -> Command {
    let mut command = Command::new("nix");
    command.arg("run").arg("github:vic/nix-versions").arg("--");
    command
}

/// Splits `nixpkgs/<commit>#<attr>` into its parts.
fn parse_installable(spec: &str, installable: &str) -> Result<ParsedInstallable, NixError> {
    let rest = installable.strip_prefix("nixpkgs/").ok_or_else(|| {
        version_resolution_error(
            spec,
            format!(
                "unexpected installable format (missing nixpkgs/ prefix): {}",
                installable
            ),
        )
    })?;
    let (commit, attr) = rest
        .split_once('#')
        .filter(|(commit, _)| !commit.is_empty())
        .ok_or_else(|| {
            version_resolution_error(
                spec,
                format!(
                    "unexpected installable format (missing # separator or empty commit): {}",
                    installable
                ),
            )
        })?;
    Ok(ParsedInstallable {
        commit: commit.to_string(),
        attr: attr.to_string(),
    })
}

/// Parses nix-versions' `--json` `stdout`, expands the resolved
/// short commit via [`devbox::expand_to_full_sha`], and assembles the
/// `ResolvedVersion` for `spec`.
///
/// Separated from [`NixVersionsResolver::resolve`] so the parse →
/// expand → assemble pipeline is testable without a subprocess, the
/// same injectable shape as [`devbox`]'s `expand_with`.
fn assemble_resolved_version(
    spec: &str,
    name: &PackageName,
    version: &PackageVersion,
    stdout: &str,
) -> Result<ResolvedVersion, NixError> {
    let entries: Vec<NixVersionEntry> = serde_json::from_str(stdout).map_err(|e| {
        version_resolution_error(spec, format!("failed to parse nix-versions output: {}", e))
    })?;
    let entry = entries.into_iter().next().ok_or_else(|| {
        version_resolution_error(
            spec,
            format!("no version found. Use 'lnix search {}' first", name),
        )
    })?;

    let parsed = parse_installable(spec, &entry.installable)?;
    let commit = devbox::expand_to_full_sha(spec, name, version, &parsed.commit)?;
    Ok(ResolvedVersion {
        commit,
        attr: parsed.attr,
    })
}

impl VersionResolver for NixVersionsResolver {
    fn resolve(
        &self,
        name: &PackageName,
        version: &PackageVersion,
    ) -> Result<ResolvedVersion, NixError> {
        let spec = format!("{}@{}", name, version);
        let mut command = nix_versions();
        command.arg("--json").arg("--one").arg(&spec);

        let captured = run_capture(command)?;
        ensure_success(&spec, &captured)?;

        assemble_resolved_version(&spec, name, version, &captured.stdout)
    }

    fn search(
        &self,
        name: &PackageName,
        version_constraint: Option<&str>,
        as_json: bool,
        only_first: bool,
    ) -> Result<String, NixError> {
        let spec = match version_constraint {
            Some(constraint) => format!("{}@{}", name, constraint),
            None => name.to_string(),
        };

        let mut command = nix_versions();
        command.arg(if as_json { "--json" } else { "--text" });
        if only_first {
            command.arg("--one");
        }
        command.arg(&spec);

        let captured = run_capture(command)?;
        ensure_success(&spec, &captured)?;
        Ok(captured.stdout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_installable() {
        let resolved = parse_installable("go@1.21.13", "nixpkgs/5ed6275#go_1_21").unwrap();

        assert_eq!(resolved.commit, "5ed6275");
        assert_eq!(resolved.attr, "go_1_21");
    }

    #[test]
    fn rejects_installable_without_nixpkgs_prefix() {
        let result = parse_installable("go@1.21.13", "invalid/5ed6275#go_1_21");

        assert!(matches!(result, Err(NixError::VersionResolution { .. })));
    }

    #[test]
    fn rejects_installable_without_attr_separator() {
        let result = parse_installable("go@1.21.13", "nixpkgs/5ed6275");

        assert!(matches!(result, Err(NixError::VersionResolution { .. })));
    }

    #[test]
    fn rejects_installable_with_empty_commit_segment() {
        let result = parse_installable("go@1.21.13", "nixpkgs/#go_1_21");

        assert!(matches!(result, Err(NixError::VersionResolution { .. })));
    }

    #[test]
    fn assembles_resolved_version_preserving_attr_through_expansion() {
        let name: PackageName = "go".parse().unwrap();
        let version: PackageVersion = "1.21.13".parse().unwrap();
        let full = "5ed62751abf1dac19bf39d6ff8c07c8c37e81e0e";
        let stdout = format!(r#"[{{"installable":"nixpkgs/{}#go_1_21"}}]"#, full);

        let resolved = assemble_resolved_version("go@1.21.13", &name, &version, &stdout).unwrap();

        assert_eq!(resolved.commit, full);
        assert_eq!(resolved.attr, "go_1_21");
    }
}
