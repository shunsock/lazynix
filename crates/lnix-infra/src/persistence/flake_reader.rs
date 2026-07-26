//! Filesystem-backed [`FlakeReader`].

use std::collections::HashMap;
use std::fs;
use std::io;

use lnix_domain::FlakeError;
use lnix_domain::interface::persistence::{FlakeReader, PinnedResolutions};
use lnix_domain::{PackageName, PackageVersion};

use crate::paths::WorkspacePaths;

const URL_PREFIX: &str = "nixpkgs--";
const URL_MID: &str = ".url";
const COMMIT_PREFIX: &str = "github:NixOS/nixpkgs/";
const ATTR_PREFIX: &str = "pinnedPkgs-";

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
        let contents = match fs::read_to_string(self.paths.flake_file()) {
            Ok(contents) => contents,
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(HashMap::new()),
            Err(err) => return Err(FlakeError::Read(err)),
        };
        Ok(parse_pinned_inputs(&contents))
    }
}

fn parse_pinned_inputs(contents: &str) -> PinnedResolutions {
    let commits = collect_commits(contents);
    let mut out = HashMap::new();
    for line in contents.lines() {
        let Some((key, attr)) = parse_attr_line(line, &commits) else {
            continue;
        };
        let commit = commits[&key].clone();
        out.insert(key, (commit, attr));
    }
    out
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
    let rest = line.trim_start().strip_prefix(URL_PREFIX)?;
    let (name_and_version, tail) = rest.split_once(URL_MID)?;
    let (name_str, dashed_version) = name_and_version.split_once("--")?;
    let url = extract_quoted_url(tail)?;
    let commit = url.strip_prefix(COMMIT_PREFIX)?;
    if commit.is_empty() {
        return None;
    }
    let key = build_key(name_str, dashed_version)?;
    Some((key, commit.to_string()))
}

fn extract_quoted_url(tail: &str) -> Option<&str> {
    let after_eq = tail.trim_start().strip_prefix('=')?.trim_start();
    let inside = after_eq.strip_prefix('"')?;
    let (url, _) = inside.split_once('"')?;
    Some(url)
}

fn parse_attr_line(
    line: &str,
    known: &HashMap<(PackageName, PackageVersion), String>,
) -> Option<((PackageName, PackageVersion), String)> {
    let trimmed = line.trim();
    let rest = trimmed.strip_prefix(ATTR_PREFIX)?;
    let (name_and_version, attr_tail) = rest.split_once('.')?;
    let key = match_known_key(name_and_version, known)?;
    let attr = attr_tail.trim_end_matches(|c: char| c == ',' || c.is_whitespace());
    if attr.is_empty() {
        return None;
    }
    Some((key, attr.to_string()))
}

fn match_known_key(
    name_and_version: &str,
    known: &HashMap<(PackageName, PackageVersion), String>,
) -> Option<(PackageName, PackageVersion)> {
    for (name, version) in known.keys() {
        let expected = format!("{}-{}", name.as_str(), version.as_str().replace('.', "-"));
        if expected == name_and_version {
            return Some((name.clone(), version.clone()));
        }
    }
    None
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
        expected.insert(
            key("go", "1.21.13"),
            ("5ed6275".to_string(), "go_1_21".to_string()),
        );
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
        expected.insert(
            key("go", "1.21.13"),
            ("5ed6275".to_string(), "go_1_21".to_string()),
        );
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
    fn reads_multiple_pinned_entries() {
        let dir = TempDir::new().unwrap();
        write_flake(&dir, TWO_PINNED_FLAKE);

        let inputs = reader_for(&dir).read_pinned_inputs().unwrap();

        let mut expected = HashMap::new();
        expected.insert(
            key("go", "1.21.13"),
            ("5ed6275".to_string(), "go_1_21".to_string()),
        );
        expected.insert(
            key("rust", "1.70.0"),
            ("abcd123".to_string(), "rustc".to_string()),
        );
        assert_eq!(inputs, expected);
    }
}
