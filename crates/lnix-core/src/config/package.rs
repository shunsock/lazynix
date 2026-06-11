use serde::{Deserialize, Serialize};

use crate::values::{PackageName, PackageVersion};

/// Packages requested for the dev shell, grouped by nixpkgs channel.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Package {
    #[serde(default)]
    pub stable: Vec<PackageEntry>,

    #[serde(default)]
    pub unstable: Vec<PackageEntry>,

    #[serde(default)]
    pub pinned: Vec<PinnedPackageEntry>,
}

/// A package resolved from a channel (stable or unstable).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageEntry {
    pub name: PackageName,
}

/// A package pinned to a specific version via nix-versions.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PinnedPackageEntry {
    pub name: PackageName,
    pub version: PackageVersion,

    /// nixpkgs commit hash. Auto-resolved via nix-versions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_commit: Option<String>,

    /// Nix attribute path (e.g., "go_1_21"). Auto-resolved via nix-versions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_attr: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_pinned_entry_with_resolution() {
        // Arrange
        let yaml = r#"
name: go
version: "1.21.13"
resolvedCommit: "5ed6275"
resolvedAttr: "go_1_21"
"#;

        // Act
        let pinned: PinnedPackageEntry = serde_yaml::from_str(yaml).unwrap();

        // Assert
        assert_eq!(pinned.name.as_str(), "go");
        assert_eq!(pinned.version.as_str(), "1.21.13");
        assert_eq!(pinned.resolved_commit.as_deref(), Some("5ed6275"));
        assert_eq!(pinned.resolved_attr.as_deref(), Some("go_1_21"));
    }

    #[test]
    fn deserializes_unresolved_pinned_entry() {
        // Arrange
        let yaml = r#"
name: go
version: "1.21.13"
"#;

        // Act
        let pinned: PinnedPackageEntry = serde_yaml::from_str(yaml).unwrap();

        // Assert
        assert!(pinned.resolved_commit.is_none());
        assert!(pinned.resolved_attr.is_none());
    }

    #[test]
    fn rejects_invalid_package_name_at_parse_time() {
        // Arrange
        let yaml = r#"
name: "invalid package!"
version: "1.0"
"#;

        // Act
        let result = serde_yaml::from_str::<PinnedPackageEntry>(yaml);

        // Assert
        let message = result.unwrap_err().to_string();
        assert!(message.contains("Invalid package name"), "got: {message}");
    }

    #[test]
    fn rejects_empty_pinned_version_at_parse_time() {
        // Arrange
        let yaml = r#"
name: go
version: ""
"#;

        // Act
        let result = serde_yaml::from_str::<PinnedPackageEntry>(yaml);

        // Assert
        let message = result.unwrap_err().to_string();
        assert!(
            message.contains("version cannot be empty"),
            "got: {message}"
        );
    }
}
