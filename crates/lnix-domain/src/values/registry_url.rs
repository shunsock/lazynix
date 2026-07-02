use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::ParseError;

/// A flake registry URL overriding the stable nixpkgs input,
/// such as `github:NixOS/nixpkgs/nixos-25.06`.
///
/// Invariant: `github:OWNER/REPO/BRANCH` where each part is non-empty
/// and contains only alphanumerics, `-`, `_`, and `.`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct RegistryUrl(String);

impl RegistryUrl {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn is_valid_registry_url(url: &str) -> bool {
    let Some(rest) = url.strip_prefix("github:") else {
        return false;
    };
    let parts: Vec<&str> = rest.split('/').collect();
    // Exactly OWNER/REPO/BRANCH
    if parts.len() != 3 {
        return false;
    }
    parts.iter().all(|part| {
        !part.is_empty()
            && part
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
    })
}

impl TryFrom<String> for RegistryUrl {
    type Error = ParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if !is_valid_registry_url(&value) {
            return Err(ParseError::InvalidRegistryUrl(value));
        }
        Ok(Self(value))
    }
}

impl FromStr for RegistryUrl {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s.to_string())
    }
}

impl From<RegistryUrl> for String {
    fn from(url: RegistryUrl) -> Self {
        url.0
    }
}

impl fmt::Display for RegistryUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_github_owner_repo_branch_form() {
        // Arrange
        let valid_urls = [
            "github:NixOS/nixpkgs/nixos-25.06",
            "github:NixOS/nixpkgs/nixos-unstable",
            "github:myuser/nixpkgs/custom-branch",
            "github:my_user/nix_pkgs/branch.name",
        ];

        // Act & Assert
        for url in valid_urls {
            assert!(url.parse::<RegistryUrl>().is_ok(), "should accept {url}");
        }
    }

    #[test]
    fn rejects_malformed_urls() {
        // Arrange
        let invalid_urls = [
            "",
            "https://github.com/NixOS/nixpkgs",
            "gitlab:NixOS/nixpkgs/main",
            "github:NixOS/nixpkgs",
            "github:NixOS/nixpkgs/branch/extra",
            "github://nixpkgs/branch",
            "github:NixOS//branch",
            "github:NixOS/nixpkgs/",
            "github:NixOS/nixpkgs/branch@tag",
            "github:Nix OS/nixpkgs/branch",
        ];

        // Act & Assert
        for url in invalid_urls {
            assert!(url.parse::<RegistryUrl>().is_err(), "should reject {url:?}");
        }
    }
}
