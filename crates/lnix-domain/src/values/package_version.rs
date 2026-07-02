use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::ParseError;

/// A version requested for a pinned package, such as `1.21.13`.
///
/// Invariant: non-empty.
///
/// The format is intentionally loose because the value is forwarded to
/// `nix-versions`, which accepts both exact versions and semver
/// constraints (e.g. `>=1.20,<1.22`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct PackageVersion(String);

impl PackageVersion {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for PackageVersion {
    type Error = ParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(ParseError::EmptyPackageVersion);
        }
        Ok(Self(value))
    }
}

impl FromStr for PackageVersion {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s.to_string())
    }
}

impl From<PackageVersion> for String {
    fn from(version: PackageVersion) -> Self {
        version.0
    }
}

impl fmt::Display for PackageVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_exact_versions_and_constraints() {
        // Arrange
        let valid_versions = ["1.21.13", "3.12", ">=1.20,<1.22"];

        // Act & Assert
        for version in valid_versions {
            assert!(
                version.parse::<PackageVersion>().is_ok(),
                "should accept {version}"
            );
        }
    }

    #[test]
    fn rejects_empty_version() {
        // Arrange
        let empty = "";

        // Act
        let result = empty.parse::<PackageVersion>();

        // Assert
        assert_eq!(result.unwrap_err(), ParseError::EmptyPackageVersion);
    }
}
