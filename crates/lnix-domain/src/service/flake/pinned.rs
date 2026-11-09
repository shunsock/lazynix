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

use crate::{DevShellDefinition, PackageName, PackageVersion, PinnedPackageEntry};

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

fn replace_dots_with_hyphens(version: &str) -> String {
    version.replace('.', "-")
}

/// Encodes (name, version) into the flake input name
/// `nixpkgs--<name>--<dashed-version>`.
///
/// CONSTRAINT: the reader (lnix_infra flake_reader) must derive names
/// via this function instead of decoding them back.
/// REASON: the encoding is non-injective; distinct versions can collide.
/// Config validation rejects pinned entries that would collide.
pub fn input_name(name: &PackageName, version: &PackageVersion) -> String {
    let dashed_version = replace_dots_with_hyphens(version.as_str());
    format!("{}{}--{}", PINNED_INPUT_PREFIX, name, dashed_version)
}

/// Encodes (name, version) into the `let`-binding / `buildInputs` name
/// `pinnedPkgs-<name>-<dashed-version>`.
///
/// See `input_name` for the shared constraint.
pub fn binding_name(name: &PackageName, version: &PackageVersion) -> String {
    let dashed_version = replace_dots_with_hyphens(version.as_str());
    format!("{}{}-{}", PINNED_BINDING_PREFIX, name, dashed_version)
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

/// Joins already-rendered lines with newlines, the shared join step of
/// every `render_*` function below.
fn join_lines(lines: Vec<String>) -> String {
    lines.join("\n")
}

fn render_pinned_build_input(entry: &&PinnedPackageEntry) -> String {
    let binding = binding_name(&entry.name, &entry.version);
    let attr = entry.resolved_attr.as_ref();
    let attr = attr.expect("filtered by collect_resolved");
    format!("            {}.{}", binding, attr)
}

/// `buildInputs` entries, e.g. `pinnedPkgs-go-1-21-13.go_1_21`.
pub(super) fn render_pinned_build_inputs(resolved: &[&PinnedPackageEntry]) -> String {
    let lines: Vec<String> = resolved.iter().map(render_pinned_build_input).collect();
    join_lines(lines)
}

fn render_pinned_input(entry: &&PinnedPackageEntry) -> String {
    let input = input_name(&entry.name, &entry.version);
    let commit = entry.resolved_commit.as_ref();
    let commit = commit.expect("filtered by collect_resolved");
    let url = format!("{}{}", PINNED_URL_COMMIT_PREFIX, commit);
    format!("    {}{} \"{}\";", input, PINNED_INPUT_URL_SUFFIX, url)
}

/// Flake `inputs` lines pinning each package to a nixpkgs revision.
pub(super) fn render_inputs(resolved: &[&PinnedPackageEntry]) -> String {
    let lines: Vec<String> = resolved.iter().map(render_pinned_input).collect();
    join_lines(lines)
}

fn render_output_param(entry: &&PinnedPackageEntry) -> String {
    input_name(&entry.name, &entry.version)
}

/// Trailing output parameters, e.g. `, nixpkgs--go--1-21-13`.
pub(super) fn render_output_params(resolved: &[&PinnedPackageEntry]) -> String {
    if resolved.is_empty() {
        return String::new();
    }
    let names: Vec<String> = resolved.iter().map(render_output_param).collect();
    format!(", {}", names.join(", "))
}

fn render_let_binding(entry: &&PinnedPackageEntry, allow_unfree: bool) -> String {
    let binding = binding_name(&entry.name, &entry.version);
    let input = input_name(&entry.name, &entry.version);
    format!(
        "        {} = import {} {{\n          inherit system;\n          config.allowUnfree = {};\n        }};",
        binding, input, allow_unfree
    )
}

/// `let` bindings importing each pinned revision with the unfree flag.
pub(super) fn render_let_bindings(resolved: &[&PinnedPackageEntry], allow_unfree: bool) -> String {
    let render = |e: &&PinnedPackageEntry| render_let_binding(e, allow_unfree);
    let lines: Vec<String> = resolved.iter().map(render).collect();
    join_lines(lines)
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
        assert_eq!(replace_dots_with_hyphens("1.21.13"), "1-21-13");
    }

    #[test]
    fn derives_input_and_binding_names() {
        let entry = resolved_go();
        let input = input_name(&entry.name, &entry.version);
        let binding = binding_name(&entry.name, &entry.version);

        assert_eq!(input, "nixpkgs--go--1-21-13");
        assert_eq!(binding, "pinnedPkgs-go-1-21-13");
    }

    #[test]
    fn encodes_hyphenated_version() {
        let name: PackageName = "dotnet-sdk_11".parse().unwrap();
        let version: PackageVersion = "11.0.100-preview.7.26381.103".parse().unwrap();
        let input = input_name(&name, &version);
        let binding = binding_name(&name, &version);

        assert_eq!(
            input,
            "nixpkgs--dotnet-sdk_11--11-0-100-preview-7-26381-103"
        );
        assert_eq!(
            binding,
            "pinnedPkgs-dotnet-sdk_11-11-0-100-preview-7-26381-103"
        );
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
