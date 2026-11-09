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
    PINNED_BINDING_PREFIX, PINNED_INPUT_PREFIX, PINNED_INPUT_URL_SUFFIX, PINNED_URL_COMMIT_PREFIX,
    binding_name, input_name,
};
use lnix_domain::{CommitSha, FlakeError, PackageName, PackageVersion};

use crate::paths::WorkspacePaths;

/// A `(name, version)` request for a pinned package.
type PinnedKey = (PackageName, PackageVersion);

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
        pinned_keys: &[PinnedKey],
    ) -> Result<PinnedResolutions, FlakeError> {
        let Ok(contents) = fs::read_to_string(self.paths.flake_file()) else {
            return Ok(HashMap::new());
        };
        Ok(parse_pinned_inputs(&contents, pinned_keys))
    }
}

/// Re-encodes each `pinned_keys` entry with the writer's naming scheme
/// and looks for the matching lines, rather than decoding the flake's
/// own names.
///
/// Walks `contents` once, matching each line against a reverse lookup
/// built from `pinned_keys` (encoded name -> key) instead of scanning
/// the file once per key; a cheap prefix check skips every line that
/// is not part of the pinned section before the per-line candidate
/// match runs. Precondition: `pinned_keys` passed `validate_config`,
/// so no two keys share an encoded input or binding name.
fn parse_pinned_inputs(contents: &str, pinned_keys: &[PinnedKey]) -> PinnedResolutions {
    let by_input = encode_lookup(pinned_keys, input_name);
    let by_binding = encode_lookup(pinned_keys, binding_name);

    let mut commits: HashMap<PinnedKey, CommitSha> = HashMap::new();
    let mut attrs: HashMap<PinnedKey, String> = HashMap::new();
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(PINNED_INPUT_PREFIX)
            && let Some((key, commit)) = longest_prefix_match(trimmed, &by_input, parse_commit)
        {
            commits.entry(key.clone()).or_insert(commit);
        } else if trimmed.starts_with(PINNED_BINDING_PREFIX)
            && let Some((key, attr)) = longest_prefix_match(trimmed, &by_binding, parse_attr)
        {
            attrs.entry(key.clone()).or_insert(attr);
        }
    }

    pinned_keys
        .iter()
        .filter_map(|key| {
            let commit = commits.get(key)?.clone();
            let attr = attrs.get(key)?.clone();
            Some((key.clone(), PinnedResolution { commit, attr }))
        })
        .collect()
}

/// Builds a reverse lookup from each key's encoded name (as produced
/// by `encode`) back to the key itself.
fn encode_lookup(
    pinned_keys: &[PinnedKey],
    encode: impl Fn(&PackageName, &PackageVersion) -> String,
) -> HashMap<String, PinnedKey> {
    pinned_keys
        .iter()
        .map(|(name, version)| (encode(name, version), (name.clone(), version.clone())))
        .collect()
}

/// Finds the candidate in `candidates` whose encoded name is the
/// longest prefix of `line` for which `parse_rest` can parse the
/// remainder.
///
/// Longest-prefix (rather than first-match) matters because one
/// entry's encoded name can be a literal prefix of another's: a
/// `PackageName` may contain a dot, so e.g. `pinnedPkgs-a-b` is a
/// prefix of `pinnedPkgs-a-b.rest-0`. Picking the longest match
/// attributes each line to the entry that actually encodes it.
fn longest_prefix_match<'a, R>(
    line: &str,
    candidates: &'a HashMap<String, PinnedKey>,
    parse_rest: impl Fn(&str) -> Option<R>,
) -> Option<(&'a PinnedKey, R)> {
    candidates
        .iter()
        .filter_map(|(encoded, key)| {
            let rest = line.strip_prefix(encoded.as_str())?;
            let parsed = parse_rest(rest)?;
            Some((encoded.len(), key, parsed))
        })
        .max_by_key(|(len, _, _)| *len)
        .map(|(_, key, parsed)| (key, parsed))
}

/// Parses the commit out of the `.url = "github:NixOS/nixpkgs/<commit>"`
/// remainder after an `<input>` match.
///
/// A commit that fails to parse as a [`CommitSha`] (e.g. nix-versions'
/// 7-digit short SHA, left over from before full-SHA expansion was
/// required) is dropped rather than cached, so the caller re-resolves
/// it from scratch instead of pinning an input to an ambiguous rev.
fn parse_commit(rest: &str) -> Option<CommitSha> {
    let after_suffix = rest.strip_prefix(PINNED_INPUT_URL_SUFFIX)?.trim_start();
    let quoted_value = after_suffix.strip_prefix('"')?;
    let (url, _) = quoted_value.split_once('"')?;
    let commit = url.strip_prefix(PINNED_URL_COMMIT_PREFIX)?;
    commit.parse::<CommitSha>().ok()
}

// NOTE: validate_config rejects collisions; the reader assumes
// NOTE: uniqueness and keeps the first matching line per key.
/// Parses the attribute out of the `.<attr>` remainder after a
/// `<binding>` match.
fn parse_attr(rest: &str) -> Option<String> {
    let after_dot = rest.strip_prefix('.')?;
    let attr: String = after_dot
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .collect();
    (!attr.is_empty()).then_some(attr)
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

    const BROKEN_AND_GO_FLAKE: &str = r#"
nixpkgs--broken--0-0-0.url = "invalid";
nixpkgs--go--1-21-13.url = "github:NixOS/nixpkgs/5ed62751abf1dac19bf39d6ff8c07c8c37e81e0e";
pinnedPkgs-broken-0-0-0.some_attr
pinnedPkgs-go-1-21-13.go_1_21
"#;

    const NO_PINNED_SECTION_FLAKE: &str = r#"
nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
nixpkgs-unstable.url = "github:NixOS/nixpkgs/nixos-unstable";
flake-utils.url = "github:numtide/flake-utils";
"#;

    const URL_ONLY_FLAKE: &str = r#"
nixpkgs--go--1-21-13.url = "github:NixOS/nixpkgs/5ed62751abf1dac19bf39d6ff8c07c8c37e81e0e";
"#;

    const ATTR_ONLY_FLAKE: &str = r#"
pinnedPkgs-go-1-21-13.go_1_21
"#;

    const URL_HELPER_FLAKE: &str = r#"
nixpkgs--urlHelper--1-0-0.url = "github:NixOS/nixpkgs/deadbee1abf1dac19bf39d6ff8c07c8c37e81e0e";
pinnedPkgs-urlHelper-1-0-0.urlHelper
"#;

    const SHORT_REV_FLAKE: &str = r#"
nixpkgs--go--1-21-13.url = "github:NixOS/nixpkgs/5ed6275";
pinnedPkgs-go-1-21-13.go_1_21
"#;

    const TRAILING_SEMICOLON_FLAKE: &str = r#"
nixpkgs--go--1-21-13.url = "github:NixOS/nixpkgs/5ed62751abf1dac19bf39d6ff8c07c8c37e81e0e";
pinnedPkgs-go-1-21-13.go_1_21;
"#;

    // NOTE: "a-b.rest" makes `pinnedPkgs-a-b` a literal prefix of
    // NOTE: `pinnedPkgs-a-b.rest-0`; the colliding line comes first.
    const PREFIX_COLLIDING_BINDING_FLAKE: &str = r#"
nixpkgs--a--b.url = "github:NixOS/nixpkgs/5ed62751abf1dac19bf39d6ff8c07c8c37e81e0e";
nixpkgs--a-b.rest--0.url = "github:NixOS/nixpkgs/abcd1231abf1dac19bf39d6ff8c07c8c37e81e0e";
pinnedPkgs-a-b.rest-0.second
pinnedPkgs-a-b.first
"#;

    const DUPLICATE_GO_LINES_FLAKE: &str = r#"
nixpkgs--go--1-21-13.url = "github:NixOS/nixpkgs/5ed62751abf1dac19bf39d6ff8c07c8c37e81e0e";
nixpkgs--go--1-21-13.url = "github:NixOS/nixpkgs/abcd1231abf1dac19bf39d6ff8c07c8c37e81e0e";
pinnedPkgs-go-1-21-13.go_1_21
pinnedPkgs-go-1-21-13.go_1_21_second
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
        write_flake(&dir, BROKEN_AND_GO_FLAKE);

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
        write_flake(&dir, NO_PINNED_SECTION_FLAKE);

        let inputs = reader_for(&dir)
            .read_pinned_inputs(&known(&[("go", "1.21.13")]))
            .unwrap();

        assert!(inputs.is_empty());
    }

    #[test]
    fn drops_entry_when_only_url_line_present() {
        let dir = TempDir::new().unwrap();
        write_flake(&dir, URL_ONLY_FLAKE);

        let inputs = reader_for(&dir)
            .read_pinned_inputs(&known(&[("go", "1.21.13")]))
            .unwrap();

        assert!(inputs.is_empty());
    }

    #[test]
    fn drops_entry_when_only_attr_line_present() {
        let dir = TempDir::new().unwrap();
        write_flake(&dir, ATTR_ONLY_FLAKE);

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
    fn resolves_each_entry_when_one_binding_name_is_a_prefix_of_another() {
        let dir = TempDir::new().unwrap();
        write_flake(&dir, PREFIX_COLLIDING_BINDING_FLAKE);

        let inputs = reader_for(&dir)
            .read_pinned_inputs(&known(&[("a", "b"), ("a-b.rest", "0")]))
            .unwrap();

        let mut expected = HashMap::new();
        expected.insert(key("a", "b"), resolution(SHA_GO, "first"));
        expected.insert(key("a-b.rest", "0"), resolution(SHA_RUST, "second"));
        assert_eq!(inputs, expected);
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
        write_flake(&dir, URL_HELPER_FLAKE);

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
        write_flake(&dir, SHORT_REV_FLAKE);

        let inputs = reader_for(&dir)
            .read_pinned_inputs(&known(&[("go", "1.21.13")]))
            .unwrap();

        assert!(inputs.is_empty());
    }

    #[test]
    fn attr_trailing_semicolon_is_stripped() {
        let dir = TempDir::new().unwrap();
        write_flake(&dir, TRAILING_SEMICOLON_FLAKE);

        let inputs = reader_for(&dir)
            .read_pinned_inputs(&known(&[("go", "1.21.13")]))
            .unwrap();

        let mut expected = HashMap::new();
        expected.insert(key("go", "1.21.13"), resolution(SHA_GO, "go_1_21"));
        assert_eq!(inputs, expected);
    }

    #[test]
    fn duplicate_lines_for_the_same_key_keep_the_first() {
        let dir = TempDir::new().unwrap();
        write_flake(&dir, DUPLICATE_GO_LINES_FLAKE);

        let inputs = reader_for(&dir)
            .read_pinned_inputs(&known(&[("go", "1.21.13")]))
            .unwrap();

        let mut expected = HashMap::new();
        expected.insert(key("go", "1.21.13"), resolution(SHA_GO, "go_1_21"));
        assert_eq!(inputs, expected);
    }
}
