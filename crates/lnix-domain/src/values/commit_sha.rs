use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::ParseError;

/// The full SHA of a commit in the nixpkgs repository.
///
/// Invariant: exactly 40 lowercase hex characters (`[0-9a-f]{40}`).
///
/// GitHub's commits API does not resolve abbreviated SHAs, so a commit
/// pinned into a flake input must always be the full SHA.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct CommitSha(String);

impl CommitSha {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn is_valid_commit_sha(value: &str) -> bool {
    value.len() == 40
        && value
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
}

impl TryFrom<String> for CommitSha {
    type Error = ParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if !is_valid_commit_sha(&value) {
            return Err(ParseError::InvalidCommitSha(value));
        }
        Ok(Self(value))
    }
}

impl FromStr for CommitSha {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s.to_string())
    }
}

impl From<CommitSha> for String {
    fn from(sha: CommitSha) -> Self {
        sha.0
    }
}

impl fmt::Display for CommitSha {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_full_lowercase_hex_sha() {
        let sha = "5ed62751abf1dac19bf39d6ff8c07c8c37e81e0e";

        let parsed = sha.parse::<CommitSha>().unwrap();

        assert_eq!(parsed.as_str(), sha);
    }

    #[test]
    fn rejects_malformed_shas() {
        let invalid_shas = [
            "5ed6275",
            "5ed62751abf1dac19bf39d6ff8c07c8c37e81e0",
            "5ed62751abf1dac19bf39d6ff8c07c8c37e81e0ea",
            "5ED62751ABF1DAC19BF39D6FF8C07C8C37E81E0E",
            "5ed62751abf1dac19bf39d6ff8c07c8c37e81ez0",
            "",
        ];

        for sha in invalid_shas {
            assert!(sha.parse::<CommitSha>().is_err(), "should reject {sha:?}");
        }
    }
}
