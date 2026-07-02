use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::ParseError;

/// A user-defined task name from the `task:` section of `lazynix.yaml`.
///
/// Invariants (checked at construction):
/// - non-empty
/// - only ASCII alphanumerics, `-`, and `_`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct TaskName(String);

impl TaskName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn is_valid_task_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

impl TryFrom<String> for TaskName {
    type Error = ParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if !is_valid_task_name(&value) {
            return Err(ParseError::InvalidTaskName(value));
        }
        Ok(Self(value))
    }
}

impl FromStr for TaskName {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s.to_string())
    }
}

impl From<TaskName> for String {
    fn from(name: TaskName) -> Self {
        name.0
    }
}

impl fmt::Display for TaskName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_alphanumeric_hyphen_underscore() {
        // Arrange
        let valid_names = ["sync", "my-task_123", "START"];

        // Act & Assert
        for name in valid_names {
            assert!(name.parse::<TaskName>().is_ok(), "should accept {name}");
        }
    }

    #[test]
    fn rejects_invalid_names() {
        // Arrange
        let invalid_names = ["", "invalid@task", "task name", "task.name"];

        // Act & Assert
        for name in invalid_names {
            assert!(name.parse::<TaskName>().is_err(), "should reject {name:?}");
        }
    }
}
