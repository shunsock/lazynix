//! Filesystem-backed [`FlakeReader`].
//!
//! Parses the exact line shapes emitted by `render_flake` in the
//! domain: the shared naming constants in
//! `lnix_domain::service::flake::pinned` are the single source of
//! truth for both writer and reader.
//!
//! SEE: crates/lnix-domain/src/service/flake/pinned.rs

use std::{collections::HashMap, fs};

use lnix_domain::interface::persistence::{FlakeReader, PinnedResolution, PinnedResolutions};
use lnix_domain::service::flake::pinned::{
    PINNED_INPUT_URL_SUFFIX, PINNED_URL_COMMIT_PREFIX, binding_name, input_name,
};
use lnix_domain::{CommitSha, FlakeError, PackageName, PackageVersion};

use crate::paths::WorkspacePaths;

/// Recovers pinned resolutions from the `flake.nix` at
/// [`WorkspacePaths::flake_file`].
pub struct FsFlakeReader {
    paths: WorkspacePaths,
}

impl FsFlakeReader {
    pub fn new(paths: WorkspacePaths) -> Self {
        Self { paths }
    }
}

impl FlakeReader for FsFlakeReader {
    fn read_pinned_inputs(
        &self,
        known: &[(PackageName, PackageVersion)],
    ) -> Result<PinnedResolutions, FlakeError> {
        let Ok(contents) = fs::read_to_string(self.paths.flake_file()) else {
            return Ok(HashMap::new());
        };
        Ok(parse_pinned_inputs(&contents, known))
    }
}

/// Re-encodes each `known` key with the writer's naming scheme and
/// looks for the matching lines, rather than decoding the flake's own
/// names.
fn parse_pinned_inputs(
    contents: &str,
    known: &[(PackageName, PackageVersion)],
) -> PinnedResolutions {
    let mut resolutions = HashMap::new();
    for (name, version) in known {
        let Some(commit) = find_commit(contents, &input_name(name, version)) else {
            continue;
        };
        let Some(attr) = find_attr(contents, &binding_name(name, version)) else {
            continue;
        };
        resolutions.insert(
            (name.clone(), version.clone()),
            PinnedResolution { commit, attr },
        );
    }
    resolutions
}

/// Finds the `<input>.url = "github:NixOS/nixpkgs/<commit>"` line and
/// parses its commit.
///
/// A commit that fails to parse as a [`CommitSha`] (e.g. nix-versions'
/// 7-digit short SHA, left over from before full-SHA expansion was
/// required) is dropped rather than cached, so the caller re-resolves
/// it from scratch instead of pinning an input to an ambiguous rev.
fn find_commit(contents: &str, input: &str) -> Option<CommitSha> {
    contents.lines().find_map(|line| {
        let rest = line.trim_start().strip_prefix(input)?;
        let after_suffix = rest.strip_prefix(PINNED_INPUT_URL_SUFFIX)?;
        let quoted_value = extract_quoted_url(after_suffix)?;
        let commit = quoted_value.strip_prefix(PINNED_URL_COMMIT_PREFIX)?;
        commit.parse::<CommitSha>().ok()
    })
}

fn extract_quoted_url(after_dot_url: &str) -> Option<&str> {
    let after_equals = after_dot_url.trim_start();
    let quoted_value = after_equals.strip_prefix('"')?;
    let (url, _) = quoted_value.split_once('"')?;
    Some(url)
}

// NOTE: 衝突設定の拒否は validate_config の責務。reader は一意性を仮定する。
/// Finds the `buildInputs` attribute suffix for the `<binding>.<attr>`
/// line.
fn find_attr(contents: &str, binding: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        let rest = line.trim().strip_prefix(binding)?;
        let after_dot = rest.strip_prefix('.')?;
        let attr: String = after_dot
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
            .collect();
        (!attr.is_empty()).then_some(attr)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_flake(dir: &TempDir, contents: &str) {
        fs::write(dir.path().join("flake.nix"), contents).unwrap();
    }

    fn reader_for(dir: &TempDir) -> FsFlakeReader {
        FsFlakeReader::new(WorkspacePaths::new(dir.path()))
    }

    fn key(name: &str, version: &str) -> (PackageName, PackageVersion) {
        (name.parse().unwrap(), version.parse().unwrap())
    }

    fn known(pairs: &[(&str, &str)]) -> Vec<(PackageName, PackageVersion)> {
        pairs.iter().map(|(n, v)| key(n, v)).collect()
    }

    fn resolution(commit: &str, attr: &str) -> PinnedResolution {
        PinnedResolution {
            commit: commit.parse().unwrap(),
            attr: attr.into(),
        }
    }

    // NOTE: named consts keep call sites on one rustfmt line instead
    // of wrapping on the inline 40-char hex literal.
    const SHA_GO: &str = "5ed62751abf1dac19bf39d6ff8c07c8c37e81e0e";
    const SHA_GO_TASK: &str = "3d464701abf1dac19bf39d6ff8c07c8c37e81e0e";
    const SHA_RUST: &str = "abcd1231abf1dac19bf39d6ff8c07c8c37e81e0e";
    const SHA_URL: &str = "deadbee1abf1dac19bf39d6ff8c07c8c37e81e0e";
    const SHA_DOTNET: &str = "cafefeedabf1dac19bf39d6ff8c07c8c37e81e0e";

    // NOTE: the reader matches lines by prefix, so fixtures need
    // only the matched lines, not a structurally valid flake.nix.
    const SINGLE_GO_FLAKE: &str = r#"
nixpkgs--go--1-21-13.url = "github:NixOS/nixpkgs/5ed62751abf1dac19bf39d6ff8c07c8c37e81e0e";
pinnedPkgs-go-1-21-13.go_1_21
"#;

    const HYPHENATED_ATTR_FLAKE: &str = r#"
nixpkgs--go-task--3-48-0.url = "github:NixOS/nixpkgs/3d464701abf1dac19bf39d6ff8c07c8c37e81e0e";
pinnedPkgs-go-task-3-48-0.go-task
"#;

    const TWO_PINNED_FLAKE: &str = r#"
nixpkgs--go--1-21-13.url = "github:NixOS/nixpkgs/5ed62751abf1dac19bf39d6ff8c07c8c37e81e0e";
nixpkgs--rust--1-70-0.url = "github:NixOS/nixpkgs/abcd1231abf1dac19bf39d6ff8c07c8c37e81e0e";
pinnedPkgs-go-1-21-13.go_1_21
pinnedPkgs-rust-1-70-0.rustc
"#;

    const HYPHENATED_VERSION_FLAKE: &str = r#"
nixpkgs--dotnet-sdk_11--11-0-100-preview-7-26381-103.url = "github:NixOS/nixpkgs/cafefeedabf1dac19bf39d6ff8c07c8c37e81e0e";
pinnedPkgs-dotnet-sdk_11-11-0-100-preview-7-26381-103.dotnet-sdk_11
"#;

    #[test]
    fn reads_single_pinned_entry() {
        let dir = TempDir::new().unwrap();
        write_flake(&dir, SINGLE_GO_FLAKE);

        let inputs = reader_for(&dir)
            .read_pinned_inputs(&known(&[("go", "1.21.13")]))
            .unwrap();

        let mut expected = HashMap::new();
        expected.insert(key("go", "1.21.13"), resolution(SHA_GO, "go_1_21"));
        assert_eq!(inputs, expected);
    }

    #[test]
    fn reads_attribute_containing_a_hyphen() {
        let dir = TempDir::new().unwrap();
        write_flake(&dir, HYPHENATED_ATTR_FLAKE);

        let inputs = reader_for(&dir)
            .read_pinned_inputs(&known(&[("go-task", "3.48.0")]))
            .unwrap();

        let mut expected = HashMap::new();
        expected.insert(key("go-task", "3.48.0"), resolution(SHA_GO_TASK, "go-task"));
        assert_eq!(inputs, expected);
    }

    #[test]
    fn round_trips_a_hyphenated_version_via_known_keys() {
        let dir = TempDir::new().unwrap();
        write_flake(&dir, HYPHENATED_VERSION_FLAKE);
        let pinned = [("dotnet-sdk_11", "11.0.100-preview.7.26381.103")];

        let inputs = reader_for(&dir)
            .read_pinned_inputs(&known(&pinned))
            .unwrap();

        let mut expected = HashMap::new();
        expected.insert(
            key("dotnet-sdk_11", "11.0.100-preview.7.26381.103"),
            resolution(SHA_DOTNET, "dotnet-sdk_11"),
        );
        assert_eq!(inputs, expected);
    }

    #[test]
    fn skips_malformed_url_and_keeps_valid_entry() {
        let dir = TempDir::new().unwrap();
        write_flake(
            &dir,
            r#"
nixpkgs--broken--0-0-0.url = "invalid";
nixpkgs--go--1-21-13.url = "github:NixOS/nixpkgs/5ed62751abf1dac19bf39d6ff8c07c8c37e81e0e";
pinnedPkgs-broken-0-0-0.some_attr
pinnedPkgs-go-1-21-13.go_1_21
"#,
        );

        let inputs = reader_for(&dir)
            .read_pinned_inputs(&known(&[("broken", "0.0.0"), ("go", "1.21.13")]))
            .unwrap();

        let mut expected = HashMap::new();
        expected.insert(key("go", "1.21.13"), resolution(SHA_GO, "go_1_21"));
        assert_eq!(inputs, expected);
    }

    #[test]
    fn returns_empty_map_for_flake_without_pinned_section() {
        let dir = TempDir::new().unwrap();
        write_flake(
            &dir,
            r#"
nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
nixpkgs-unstable.url = "github:NixOS/nixpkgs/nixos-unstable";
flake-utils.url = "github:numtide/flake-utils";
"#,
        );

        let inputs = reader_for(&dir)
            .read_pinned_inputs(&known(&[("go", "1.21.13")]))
            .unwrap();

        assert!(inputs.is_empty());
    }

    #[test]
    fn drops_entry_when_only_url_line_present() {
        let dir = TempDir::new().unwrap();
        write_flake(
            &dir,
            r#"
nixpkgs--go--1-21-13.url = "github:NixOS/nixpkgs/5ed62751abf1dac19bf39d6ff8c07c8c37e81e0e";
"#,
        );

        let inputs = reader_for(&dir)
            .read_pinned_inputs(&known(&[("go", "1.21.13")]))
            .unwrap();

        assert!(inputs.is_empty());
    }

    #[test]
    fn drops_entry_when_only_attr_line_present() {
        let dir = TempDir::new().unwrap();
        write_flake(
            &dir,
            r#"
pinnedPkgs-go-1-21-13.go_1_21
"#,
        );

        let inputs = reader_for(&dir)
            .read_pinned_inputs(&known(&[("go", "1.21.13")]))
            .unwrap();

        assert!(inputs.is_empty());
    }

    #[test]
    fn returns_empty_map_when_flake_missing() {
        let dir = TempDir::new().unwrap();

        let inputs = reader_for(&dir)
            .read_pinned_inputs(&known(&[("go", "1.21.13")]))
            .unwrap();

        assert!(inputs.is_empty());
    }

    #[test]
    fn returns_empty_map_when_read_fails() {
        let dir = TempDir::new().unwrap();
        fs::create_dir(dir.path().join("flake.nix")).unwrap();

        let inputs = reader_for(&dir)
            .read_pinned_inputs(&known(&[("go", "1.21.13")]))
            .unwrap();

        assert!(inputs.is_empty());
    }

    #[test]
    fn reads_multiple_pinned_entries() {
        let dir = TempDir::new().unwrap();
        write_flake(&dir, TWO_PINNED_FLAKE);

        let inputs = reader_for(&dir)
            .read_pinned_inputs(&known(&[("go", "1.21.13"), ("rust", "1.70.0")]))
            .unwrap();

        let mut expected = HashMap::new();
        expected.insert(key("go", "1.21.13"), resolution(SHA_GO, "go_1_21"));
        expected.insert(key("rust", "1.70.0"), resolution(SHA_RUST, "rustc"));
        assert_eq!(inputs, expected);
    }

    #[test]
    fn does_not_confuse_name_containing_dot_url_prefix() {
        let dir = TempDir::new().unwrap();
        write_flake(
            &dir,
            r#"
nixpkgs--urlHelper--1-0-0.url = "github:NixOS/nixpkgs/deadbee1abf1dac19bf39d6ff8c07c8c37e81e0e";
pinnedPkgs-urlHelper-1-0-0.urlHelper
"#,
        );

        let inputs = reader_for(&dir)
            .read_pinned_inputs(&known(&[("urlHelper", "1.0.0")]))
            .unwrap();

        let mut expected = HashMap::new();
        expected.insert(key("urlHelper", "1.0.0"), resolution(SHA_URL, "urlHelper"));
        assert_eq!(inputs, expected);
    }

    #[test]
    fn drops_entry_whose_url_line_has_a_short_rev() {
        let dir = TempDir::new().unwrap();
        write_flake(
            &dir,
            r#"
nixpkgs--go--1-21-13.url = "github:NixOS/nixpkgs/5ed6275";
pinnedPkgs-go-1-21-13.go_1_21
"#,
        );

        let inputs = reader_for(&dir)
            .read_pinned_inputs(&known(&[("go", "1.21.13")]))
            .unwrap();

        assert!(inputs.is_empty());
    }

    #[test]
    fn attr_trailing_semicolon_is_stripped() {
        let dir = TempDir::new().unwrap();
        write_flake(
            &dir,
            r#"
nixpkgs--go--1-21-13.url = "github:NixOS/nixpkgs/5ed62751abf1dac19bf39d6ff8c07c8c37e81e0e";
pinnedPkgs-go-1-21-13.go_1_21;
"#,
        );

        let inputs = reader_for(&dir)
            .read_pinned_inputs(&known(&[("go", "1.21.13")]))
            .unwrap();

        let mut expected = HashMap::new();
        expected.insert(key("go", "1.21.13"), resolution(SHA_GO, "go_1_21"));
        assert_eq!(inputs, expected);
    }
}
