use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::ParseError;

/// An environment variable name exported into the dev shell.
///
/// Invariant: matches `[a-zA-Z_][a-zA-Z0-9_]*` (POSIX shell variable name).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct EnvVarName(String);

impl EnvVarName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn is_valid_env_var_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_alphabetic() && first != '_' {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

impl TryFrom<String> for EnvVarName {
    type Error = ParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if !is_valid_env_var_name(&value) {
            return Err(ParseError::InvalidEnvVarName(value));
        }
        Ok(Self(value))
    }
}

impl FromStr for EnvVarName {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s.to_string())
    }
}

impl From<EnvVarName> for String {
    fn from(name: EnvVarName) -> Self {
        name.0
    }
}

impl fmt::Display for EnvVarName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_posix_variable_names() {
        // Arrange
        let valid_names = ["MY_VAR", "_PRIVATE", "PATH", "VAR123", "_", "a"];

        // Act & Assert
        for name in valid_names {
            assert!(name.parse::<EnvVarName>().is_ok(), "should accept {name}");
        }
    }

    #[test]
    fn rejects_invalid_names() {
        // Arrange
        let invalid_names = ["123VAR", "MY-VAR", "MY VAR", "MY.VAR", ""];

        // Act & Assert
        for name in invalid_names {
            assert!(
                name.parse::<EnvVarName>().is_err(),
                "should reject {name:?}"
            );
        }
    }
}
