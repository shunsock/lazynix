//! Renders the flake fragments for version-pinned packages.
//!
//! Each pinned package becomes its own flake input (a nixpkgs revision),
//! an output parameter, a `let` binding importing that revision, and a
//! `buildInputs` entry. Only packages whose version has already been
//! resolved to a commit + attribute are emitted.
//!
//! The naming constants below are also imported by the reader adapter
//! (see `lnix_infra::persistence::flake_reader`); keeping them in one
//! place prevents silent breakage when the writer format changes.

use crate::{DevShellDefinition, PinnedPackageEntry};

/// Prefix of a flake input name for a pinned package
/// (`nixpkgs--<name>--<dashed-version>`).
pub const PINNED_INPUT_PREFIX: &str = "nixpkgs--";

/// Suffix appended to an input name when it becomes a `.url = ...` line
/// in the flake, including the surrounding space and `=`.
pub const PINNED_INPUT_URL_SUFFIX: &str = ".url =";

/// Prefix of the `let`-binding / `buildInputs` name for a pinned
/// package (`pinnedPkgs-<name>-<dashed-version>`).
pub const PINNED_BINDING_PREFIX: &str = "pinnedPkgs-";

/// Prefix of the resolved `.url` value that points at a specific
/// nixpkgs commit (`github:NixOS/nixpkgs/<commit>`).
pub const PINNED_URL_COMMIT_PREFIX: &str = "github:NixOS/nixpkgs/";

fn normalize_version(version: &str) -> String {
    version.replace('.', "-")
}

fn input_name(entry: &PinnedPackageEntry) -> String {
    format!(
        "{}{}--{}",
        PINNED_INPUT_PREFIX,
        entry.name,
        normalize_version(entry.version.as_str())
    )
}

fn binding_name(entry: &PinnedPackageEntry) -> String {
    format!(
        "{}{}-{}",
        PINNED_BINDING_PREFIX,
        entry.name,
        normalize_version(entry.version.as_str())
    )
}

/// Collects the pinned entries that have been resolved to a concrete
/// nixpkgs commit and attribute. Unresolved entries are skipped.
pub(super) fn collect_resolved(config: &DevShellDefinition) -> Vec<&PinnedPackageEntry> {
    config
        .dev_shell
        .package
        .pinned
        .iter()
        .filter(|p| p.resolved_commit.is_some() && p.resolved_attr.is_some())
        .collect()
}

/// `buildInputs` entries, e.g. `pinnedPkgs-go-1-21-13.go_1_21`.
pub(super) fn render_packages(resolved: &[&PinnedPackageEntry]) -> String {
    resolved
        .iter()
        .map(|entry| {
            let attr = entry.resolved_attr.as_ref().unwrap();
            format!("            {}.{}", binding_name(entry), attr)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Flake `inputs` lines pinning each package to a nixpkgs revision.
pub(super) fn render_inputs(resolved: &[&PinnedPackageEntry]) -> String {
    resolved
        .iter()
        .map(|entry| {
            let commit = entry.resolved_commit.as_ref().unwrap();
            format!(
                "    {}.url = \"github:NixOS/nixpkgs/{}\";",
                input_name(entry),
                commit
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Trailing output parameters, e.g. `, nixpkgs--go--1-21-13`.
pub(super) fn render_output_params(resolved: &[&PinnedPackageEntry]) -> String {
    if resolved.is_empty() {
        return String::new();
    }
    let params = resolved
        .iter()
        .map(|entry| input_name(entry))
        .collect::<Vec<_>>()
        .join(", ");
    format!(", {}", params)
}

/// `let` bindings importing each pinned revision with the unfree flag.
pub(super) fn render_let_bindings(resolved: &[&PinnedPackageEntry], allow_unfree: bool) -> String {
    resolved
        .iter()
        .map(|entry| {
            format!(
                "        {} = import {} {{\n          inherit system;\n          config.allowUnfree = {};\n        }};",
                binding_name(entry),
                input_name(entry),
                allow_unfree
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resolved_go() -> PinnedPackageEntry {
        serde_yaml::from_str(
            r#"
name: go
version: "1.21.13"
resolvedCommit: "e607cb5"
resolvedAttr: "go_1_21"
"#,
        )
        .unwrap()
    }

    #[test]
    fn normalizes_dots_to_hyphens() {
        assert_eq!(normalize_version("1.21.13"), "1-21-13");
    }

    #[test]
    fn derives_input_and_binding_names() {
        let entry = resolved_go();

        assert_eq!(input_name(&entry), "nixpkgs--go--1-21-13");
        assert_eq!(binding_name(&entry), "pinnedPkgs-go-1-21-13");
    }

    #[test]
    fn renders_input_pinned_to_commit() {
        let entry = resolved_go();

        let inputs = render_inputs(&[&entry]);

        assert_eq!(
            inputs,
            "    nixpkgs--go--1-21-13.url = \"github:NixOS/nixpkgs/e607cb5\";"
        );
    }

    #[test]
    fn output_params_are_empty_without_resolved_entries() {
        assert_eq!(render_output_params(&[]), "");
    }
}
