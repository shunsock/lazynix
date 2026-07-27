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
};
use lnix_domain::{FlakeError, PackageName, PackageVersion};

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
    fn read_pinned_inputs(&self) -> Result<PinnedResolutions, FlakeError> {
        let Ok(contents) = fs::read_to_string(self.paths.flake_file()) else {
            return Ok(HashMap::new());
        };
        Ok(parse_pinned_inputs(&contents))
    }
}

fn parse_pinned_inputs(contents: &str) -> PinnedResolutions {
    let mut commits = collect_commits(contents);
    let mut resolutions = HashMap::new();
    for line in contents.lines() {
        let Some((key, attr)) = parse_attr_line(line, &commits) else {
            continue;
        };
        let commit = commits
            .remove(&key)
            .expect("commit present by construction");
        resolutions.insert(key, PinnedResolution { commit, attr });
    }
    resolutions
}

fn collect_commits(contents: &str) -> HashMap<(PackageName, PackageVersion), String> {
    let mut commits = HashMap::new();
    for line in contents.lines() {
        if let Some((key, commit)) = parse_url_line(line) {
            commits.insert(key, commit);
        }
    }
    commits
}

fn parse_url_line(line: &str) -> Option<((PackageName, PackageVersion), String)> {
    let after_input_prefix = line.trim_start().strip_prefix(PINNED_INPUT_PREFIX)?;
    let (name_and_version, after_dot_url) =
        after_input_prefix.split_once(PINNED_INPUT_URL_SUFFIX)?;
    let (name_str, dashed_version) = split_name_and_version(name_and_version)?;
    let quoted_value = extract_quoted_url(after_dot_url)?;
    let commit = quoted_value.strip_prefix(PINNED_URL_COMMIT_PREFIX)?;
    if commit.is_empty() {
        return None;
    }
    let key = build_key(name_str, dashed_version)?;
    Some((key, commit.to_string()))
}

fn extract_quoted_url(after_dot_url: &str) -> Option<&str> {
    let after_equals = after_dot_url.trim_start();
    let quoted_value = after_equals.strip_prefix('"')?;
    let (url, _) = quoted_value.split_once('"')?;
    Some(url)
}

fn parse_attr_line(
    line: &str,
    commits: &HashMap<(PackageName, PackageVersion), String>,
) -> Option<((PackageName, PackageVersion), String)> {
    let trimmed = line.trim();
    let rest = trimmed.strip_prefix(PINNED_BINDING_PREFIX)?;
    let (name_and_version, attr_tail) = rest.split_once('.')?;
    let key = match_known_key(name_and_version, commits)?;
    let attr: String = attr_tail
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect();
    if attr.is_empty() {
        return None;
    }
    Some((key, attr))
}

fn match_known_key(
    name_and_version: &str,
    commits: &HashMap<(PackageName, PackageVersion), String>,
) -> Option<(PackageName, PackageVersion)> {
    for (name, version) in commits.keys() {
        let expected = format!("{}-{}", name.as_str(), version.as_str().replace('.', "-"));
        if expected == name_and_version {
            return Some((name.clone(), version.clone()));
        }
    }
    None
}

/// Splits `name-and-dashed-version` at the `--` separator emitted by
/// the writer. Callers rely on `PackageName` rejecting names that
/// themselves contain `--` so this split is unambiguous.
fn split_name_and_version(name_and_version: &str) -> Option<(&str, &str)> {
    name_and_version.split_once("--")
}

fn build_key(name_str: &str, dashed_version: &str) -> Option<(PackageName, PackageVersion)> {
    let name = name_str.parse::<PackageName>().ok()?;
    let version = dashed_version
        .replace('-', ".")
        .parse::<PackageVersion>()
        .ok()?;
    Some((name, version))
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

    fn resolution(commit: &str, attr: &str) -> PinnedResolution {
        PinnedResolution {
            commit: commit.into(),
            attr: attr.into(),
        }
    }

    const SINGLE_GO_FLAKE: &str = r#"{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    nixpkgs--go--1-21-13.url = "github:NixOS/nixpkgs/5ed6275";
  };
  outputs = { self, nixpkgs, nixpkgs--go--1-21-13, ... }:
    let
      pinnedPkgs-go-1-21-13 = import nixpkgs--go--1-21-13 { };
    in
    {
      devShells.default = stablePackages.mkShell {
        buildInputs = [
          pinnedPkgs-go-1-21-13.go_1_21
        ];
      };
    };
}
"#;

    const TWO_PINNED_FLAKE: &str = r#"{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    nixpkgs--go--1-21-13.url = "github:NixOS/nixpkgs/5ed6275";
    nixpkgs--rust--1-70-0.url = "github:NixOS/nixpkgs/abcd123";
  };
  outputs = { self, nixpkgs, nixpkgs--go--1-21-13, nixpkgs--rust--1-70-0, ... }:
    let
      pinnedPkgs-go-1-21-13 = import nixpkgs--go--1-21-13 { };
      pinnedPkgs-rust-1-70-0 = import nixpkgs--rust--1-70-0 { };
    in
    {
      devShells.default = stablePackages.mkShell {
        buildInputs = [
          pinnedPkgs-go-1-21-13.go_1_21
          pinnedPkgs-rust-1-70-0.rustc
        ];
      };
    };
}
"#;

    #[test]
    fn reads_single_pinned_entry() {
        let dir = TempDir::new().unwrap();
        write_flake(&dir, SINGLE_GO_FLAKE);

        let inputs = reader_for(&dir).read_pinned_inputs().unwrap();

        let mut expected = HashMap::new();
        expected.insert(key("go", "1.21.13"), resolution("5ed6275", "go_1_21"));
        assert_eq!(inputs, expected);
    }

    #[test]
    fn skips_malformed_url_and_keeps_valid_entry() {
        let dir = TempDir::new().unwrap();
        write_flake(
            &dir,
            r#"{
  inputs = {
    nixpkgs--broken--0-0-0.url = "invalid";
    nixpkgs--go--1-21-13.url = "github:NixOS/nixpkgs/5ed6275";
  };
  outputs = {
    devShells.default = stablePackages.mkShell {
      buildInputs = [
        pinnedPkgs-broken-0-0-0.some_attr
        pinnedPkgs-go-1-21-13.go_1_21
      ];
    };
  };
}
"#,
        );

        let inputs = reader_for(&dir).read_pinned_inputs().unwrap();

        let mut expected = HashMap::new();
        expected.insert(key("go", "1.21.13"), resolution("5ed6275", "go_1_21"));
        assert_eq!(inputs, expected);
    }

    #[test]
    fn returns_empty_map_for_flake_without_pinned_section() {
        let dir = TempDir::new().unwrap();
        write_flake(
            &dir,
            r#"{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    nixpkgs-unstable.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = { self, nixpkgs, nixpkgs-unstable, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        stablePackages = import nixpkgs { inherit system; };
      in
      {
        devShells.default = stablePackages.mkShell {
          buildInputs = [
            stablePackages.bash
          ];
        };
      });
}
"#,
        );

        let inputs = reader_for(&dir).read_pinned_inputs().unwrap();

        assert!(inputs.is_empty());
    }

    #[test]
    fn drops_entry_when_only_url_line_present() {
        let dir = TempDir::new().unwrap();
        write_flake(
            &dir,
            r#"{
  inputs = {
    nixpkgs--go--1-21-13.url = "github:NixOS/nixpkgs/5ed6275";
  };
}
"#,
        );

        let inputs = reader_for(&dir).read_pinned_inputs().unwrap();

        assert!(inputs.is_empty());
    }

    #[test]
    fn drops_entry_when_only_attr_line_present() {
        let dir = TempDir::new().unwrap();
        write_flake(
            &dir,
            r#"{
  outputs = {
    devShells.default = stablePackages.mkShell {
      buildInputs = [
        pinnedPkgs-go-1-21-13.go_1_21
      ];
    };
  };
}
"#,
        );

        let inputs = reader_for(&dir).read_pinned_inputs().unwrap();

        assert!(inputs.is_empty());
    }

    #[test]
    fn returns_empty_map_when_flake_missing() {
        let dir = TempDir::new().unwrap();

        let inputs = reader_for(&dir).read_pinned_inputs().unwrap();

        assert!(inputs.is_empty());
    }

    #[test]
    fn returns_empty_map_when_read_fails() {
        let dir = TempDir::new().unwrap();
        fs::create_dir(dir.path().join("flake.nix")).unwrap();

        let inputs = reader_for(&dir).read_pinned_inputs().unwrap();

        assert!(inputs.is_empty());
    }

    #[test]
    fn reads_multiple_pinned_entries() {
        let dir = TempDir::new().unwrap();
        write_flake(&dir, TWO_PINNED_FLAKE);

        let inputs = reader_for(&dir).read_pinned_inputs().unwrap();

        let mut expected = HashMap::new();
        expected.insert(key("go", "1.21.13"), resolution("5ed6275", "go_1_21"));
        expected.insert(key("rust", "1.70.0"), resolution("abcd123", "rustc"));
        assert_eq!(inputs, expected);
    }

    #[test]
    fn does_not_confuse_name_containing_dot_url_prefix() {
        let dir = TempDir::new().unwrap();
        write_flake(
            &dir,
            r#"{
  inputs = {
    nixpkgs--urlHelper--1-0-0.url = "github:NixOS/nixpkgs/deadbee";
  };
  outputs = { self, ... }:
    let
      pinnedPkgs-urlHelper-1-0-0 = import nixpkgs--urlHelper--1-0-0 { };
    in
    {
      devShells.default = stablePackages.mkShell {
        buildInputs = [
          pinnedPkgs-urlHelper-1-0-0.urlHelper
        ];
      };
    };
}
"#,
        );

        let inputs = reader_for(&dir).read_pinned_inputs().unwrap();

        let mut expected = HashMap::new();
        expected.insert(
            key("urlHelper", "1.0.0"),
            resolution("deadbee", "urlHelper"),
        );
        assert_eq!(inputs, expected);
    }

    #[test]
    fn attr_trailing_semicolon_is_stripped() {
        let dir = TempDir::new().unwrap();
        write_flake(
            &dir,
            r#"{
  inputs = {
    nixpkgs--go--1-21-13.url = "github:NixOS/nixpkgs/5ed6275";
  };
  outputs = { self, ... }:
    {
      devShells.default = stablePackages.mkShell {
        buildInputs = [
          pinnedPkgs-go-1-21-13.go_1_21;
        ];
      };
    };
}
"#,
        );

        let inputs = reader_for(&dir).read_pinned_inputs().unwrap();

        let mut expected = HashMap::new();
        expected.insert(key("go", "1.21.13"), resolution("5ed6275", "go_1_21"));
        assert_eq!(inputs, expected);
    }
}
