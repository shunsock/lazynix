use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::ParseError;

/// A nixpkgs attribute name such as `python312` or `python312Packages.pip`.
///
/// Invariants (checked at construction):
/// - non-empty
/// - only alphanumerics, `-`, `_`, and `.`
/// - dots may appear only between segments (no leading/trailing/double dots)
///
/// The character restriction doubles as a shell-injection guard:
/// a `PackageName` can be safely embedded in `nix` command arguments.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct PackageName(String);

impl PackageName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn is_valid_package_name(name: &str) -> bool {
    if name.is_empty() || name.starts_with('.') || name.ends_with('.') || name.contains("..") {
        return false;
    }
    name.chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
}

impl TryFrom<String> for PackageName {
    type Error = ParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if !is_valid_package_name(&value) {
            return Err(ParseError::InvalidPackageName(value));
        }
        Ok(Self(value))
    }
}

impl FromStr for PackageName {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s.to_string())
    }
}

impl From<PackageName> for String {
    fn from(name: PackageName) -> Self {
        name.0
    }
}

impl fmt::Display for PackageName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_simple_and_nested_attribute_names() {
        // Arrange
        let valid_names = [
            "python312",
            "rust-analyzer",
            "node_js",
            "python312Packages.pip",
            "lib.strings.concatStringsSep",
        ];

        // Act & Assert
        for name in valid_names {
            assert!(name.parse::<PackageName>().is_ok(), "should accept {name}");
        }
    }

    #[test]
    fn rejects_invalid_names() {
        // Arrange
        let invalid_names = [
            "",
            "pkg@version",
            "pkg with space",
            "pkg/path",
            ".pkg",
            "pkg.",
            "pkg..name",
        ];

        // Act & Assert
        for name in invalid_names {
            assert!(
                name.parse::<PackageName>().is_err(),
                "should reject {name:?}"
            );
        }
    }

    #[test]
    fn rejects_shell_metacharacters() {
        // Arrange
        let injection_attempts = ["pkg$var", "pkg;cmd", "pkg|cmd", "vim `whoami`"];

        // Act & Assert
        for name in injection_attempts {
            assert!(
                name.parse::<PackageName>().is_err(),
                "should reject {name:?}"
            );
        }
    }
}
