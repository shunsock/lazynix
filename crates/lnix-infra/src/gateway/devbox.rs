//! Devbox resolve API: expands nix-versions' 7-digit short commit SHAs
//! into the 40-digit full SHA that a pinned-input `rev` requires.

use std::process::Command;

use serde::Deserialize;

use lnix_domain::{CommitSha, NixError, PackageName, PackageVersion};

use crate::process::{Captured, run_capture};

/// Builds a `NixError::VersionResolution` for `spec` with `message`.
///
/// Shared by this module and [`super::version_resolver`] so the two
/// fields are assembled in exactly one place.
pub(crate) fn version_resolution_error(spec: &str, message: impl Into<String>) -> NixError {
    NixError::VersionResolution {
        spec: spec.to_string(),
        message: message.into(),
    }
}

/// Rejects `captured` if its subprocess exited non-zero, wrapping stderr
/// into a `NixError::VersionResolution` for `spec`.
pub(crate) fn ensure_success(spec: &str, captured: &Captured) -> Result<(), NixError> {
    if captured.success {
        return Ok(());
    }
    Err(version_resolution_error(spec, captured.stderr.clone()))
}

/// Builds `nix run nixpkgs#curl -- -fsSL -G ... https://search.devbox.sh/v1/resolve`.
///
/// `-G` + `--data-urlencode` route the query through curl's own query-string
/// encoder so that semver constraints (`>=1.20,<1.22`) and other
/// `PackageVersion` characters survive as a single safely-encoded parameter,
/// without the caller building a URL by hand.
fn curl_resolve_command(name: &PackageName, version: &PackageVersion) -> Command {
    let mut command = Command::new("nix");
    command
        .arg("run")
        .arg("nixpkgs#curl")
        .arg("--")
        .arg("-fsSL")
        .arg("--max-time")
        .arg("10")
        .arg("-G")
        .arg("--data-urlencode")
        .arg(format!("name={}", name))
        .arg("--data-urlencode")
        .arg(format!("version={}", version))
        .arg("https://search.devbox.sh/v1/resolve");
    command
}

/// The subset of the Devbox v1 resolve response this adapter reads.
#[derive(Debug, Deserialize)]
struct ResolveResponse {
    commit_hash: String,
}

/// Extracts and validates `commit_hash` from a Devbox v1 resolve
/// response body, parsing it as a [`CommitSha`] so a malformed or
/// abbreviated hash is rejected in one place.
fn parse_resolve_response(spec: &str, body: &str) -> Result<CommitSha, NixError> {
    let response: ResolveResponse = serde_json::from_str(body).map_err(|e| {
        version_resolution_error(
            spec,
            format!("failed to parse devbox resolve response: {}", e),
        )
    })?;

    response.commit_hash.parse::<CommitSha>().map_err(|_| {
        version_resolution_error(
            spec,
            format!(
                "devbox resolve returned a non-full-sha commit_hash: {}",
                response.commit_hash
            ),
        )
    })
}

/// Rejects a `full` SHA that does not extend `short` as a prefix.
///
/// NOTE: nix-versions and Devbox query distinct registries. Without
/// this check they could silently agree on different commits.
fn ensure_prefix_match(spec: &str, short: &str, full: &CommitSha) -> Result<(), NixError> {
    if !short.is_empty() && full.as_str().starts_with(short) {
        return Ok(());
    }
    Err(version_resolution_error(
        spec,
        format!(
            "devbox resolved commit {} does not extend short commit {}",
            full, short
        ),
    ))
}

/// Runs `command` to resolve `spec`'s `short_commit` to a full SHA.
fn expand_with(command: Command, spec: &str, short_commit: &str) -> Result<CommitSha, NixError> {
    if let Ok(full) = short_commit.parse::<CommitSha>() {
        return Ok(full);
    }

    let captured = run_capture(command)?;
    ensure_success(spec, &captured)?;

    let full = parse_resolve_response(spec, &captured.stdout)?;
    ensure_prefix_match(spec, short_commit, &full)?;
    Ok(full)
}

/// Expands `short_commit` (as resolved for `spec`) into its 40-digit
/// full SHA by querying the Devbox resolve API.
pub(crate) fn expand_to_full_sha(
    spec: &str,
    name: &PackageName,
    version: &PackageVersion,
    short_commit: &str,
) -> Result<CommitSha, NixError> {
    expand_with(curl_resolve_command(name, version), spec, short_commit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_nix_run_curl_command_with_devbox_query() {
        let name: PackageName = "go".parse().unwrap();
        let version: PackageVersion = "1.21.13".parse().unwrap();

        let command = curl_resolve_command(&name, &version);

        assert_eq!(command.get_program(), "nix");
        let args: Vec<_> = command.get_args().map(|a| a.to_str().unwrap()).collect();
        assert_eq!(&args[0..3], &["run", "nixpkgs#curl", "--"]);
        assert!(args.contains(&"-G"));
        assert!(args.contains(&"--data-urlencode"));
        assert!(args.contains(&"name=go"));
        assert!(args.contains(&"version=1.21.13"));
    }

    #[test]
    fn builds_curl_command_with_a_max_time_bound() {
        let name: PackageName = "go".parse().unwrap();
        let version: PackageVersion = "1.21.13".parse().unwrap();

        let command = curl_resolve_command(&name, &version);

        let args: Vec<_> = command.get_args().map(|a| a.to_str().unwrap()).collect();
        let i = args.iter().position(|a| *a == "--max-time").unwrap();
        assert_eq!(args[i + 1], "10");
    }

    #[test]
    fn parses_commit_hash_from_resolve_response() {
        let body = r#"{"commit_hash":"5ed62751abf1dac19bf39d6ff8c07c8c37e81e0e","name":"go"}"#;

        let commit = parse_resolve_response("go@1.21.13", body).unwrap();

        assert_eq!(commit.as_str(), "5ed62751abf1dac19bf39d6ff8c07c8c37e81e0e");
    }

    #[test]
    fn rejects_malformed_resolve_responses() {
        let bodies = [
            "not json",
            r#"{"name":"go"}"#,
            r#"{"commit_hash":"5ed6275"}"#,
            r#"{"commit_hash":"5ED62751ABF1DAC19BF39D6FF8C07C8C37E81E0E"}"#,
            r#"{"commit_hash":"ZZZ62751abf1dac19bf39d6ff8c07c8c37e81e0e"}"#,
        ];

        for body in bodies {
            let result = parse_resolve_response("go@1.21.13", body);
            assert!(matches!(result, Err(NixError::VersionResolution { .. })));
        }
    }

    #[test]
    fn rejects_full_sha_not_prefixed_by_short_sha() {
        let full: CommitSha = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".parse().unwrap();

        let result = ensure_prefix_match("go@1.21.13", "5ed6275", &full);

        assert!(matches!(result, Err(NixError::VersionResolution { .. })));
    }

    #[test]
    fn rejects_empty_short_commit_even_if_full_sha_would_match() {
        // NOTE: any full string trivially starts_with(""), so a
        // short placeholder suffices to exercise the guard.
        let full: CommitSha = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef".parse().unwrap();

        let result = ensure_prefix_match("go@1.21.13", "", &full);

        assert!(matches!(result, Err(NixError::VersionResolution { .. })));
    }

    #[test]
    fn expand_with_wraps_stderr_on_nonzero_exit() {
        let mut command = Command::new("sh");
        command.arg("-c").arg("echo boom 1>&2; exit 22");

        let result = expand_with(command, "go@1.21.13", "5ed6275");

        assert!(result.unwrap_err().to_string().contains("boom"));
    }

    #[test]
    fn expand_with_propagates_spawn_failure() {
        let command = Command::new("definitely-not-a-real-binary-xyz");

        let result = expand_with(command, "go@1.21.13", "5ed6275");

        assert!(matches!(result, Err(NixError::Spawn(_))));
    }

    #[test]
    fn expand_with_short_circuits_when_short_commit_is_already_full_sha() {
        let full = "5ed62751abf1dac19bf39d6ff8c07c8c37e81e0e";
        let command = Command::new("definitely-not-a-real-binary-xyz");

        let result = expand_with(command, "go@1.21.13", full);

        assert_eq!(result.unwrap().as_str(), full);
    }

    #[test]
    fn expand_with_expands_short_commit_to_prefix_matching_full_sha() {
        let full = "5ed62751abf1dac19bf39d6ff8c07c8c37e81e0e";
        let mut command = Command::new("sh");
        command
            .arg("-c")
            .arg(format!("echo '{{\"commit_hash\":\"{}\"}}'", full));

        let result = expand_with(command, "go@1.21.13", "5ed6275");

        assert_eq!(result.unwrap().as_str(), full);
    }
}
