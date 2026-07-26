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
    #[serde(default, skip_serializing_if = "always_skip_serialization")]
    pub resolved_commit: Option<String>,

    /// Nix attribute path (e.g., "go_1_21"). Auto-resolved via nix-versions.
    #[serde(default, skip_serializing_if = "always_skip_serialization")]
    pub resolved_attr: Option<String>,
}

// NOTE: resolved fields never serialize; flake.nix owns the SSoT.
fn always_skip_serialization<T>(_: &T) -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_pinned_entry_with_resolution() {
        let yaml = r#"
name: go
version: "1.21.13"
resolvedCommit: "5ed6275"
resolvedAttr: "go_1_21"
"#;

        let pinned: PinnedPackageEntry = serde_yaml::from_str(yaml).unwrap();

        assert_eq!(pinned.name.as_str(), "go");
        assert_eq!(pinned.version.as_str(), "1.21.13");
        assert_eq!(pinned.resolved_commit.as_deref(), Some("5ed6275"));
        assert_eq!(pinned.resolved_attr.as_deref(), Some("go_1_21"));
    }

    #[test]
    fn deserializes_unresolved_pinned_entry() {
        let yaml = r#"
name: go
version: "1.21.13"
"#;

        let pinned: PinnedPackageEntry = serde_yaml::from_str(yaml).unwrap();

        assert!(pinned.resolved_commit.is_none());
        assert!(pinned.resolved_attr.is_none());
    }

    #[test]
    fn rejects_invalid_package_name_at_parse_time() {
        let yaml = r#"
name: "invalid package!"
version: "1.0"
"#;

        let result = serde_yaml::from_str::<PinnedPackageEntry>(yaml);

        let message = result.unwrap_err().to_string();
        assert!(message.contains("Invalid package name"), "got: {message}");
    }

    #[test]
    fn rejects_empty_pinned_version_at_parse_time() {
        let yaml = r#"
name: go
version: ""
"#;

        let result = serde_yaml::from_str::<PinnedPackageEntry>(yaml);

        let message = result.unwrap_err().to_string();
        assert!(
            message.contains("version cannot be empty"),
            "got: {message}"
        );
    }

    #[test]
    fn serialize_omits_resolved_commit() {
        let pinned = PinnedPackageEntry {
            name: "go".parse().unwrap(),
            version: "1.21.13".parse().unwrap(),
            resolved_commit: Some("5ed6275".to_string()),
            resolved_attr: None,
        };

        let yaml = serde_yaml::to_string(&pinned).unwrap();

        assert!(
            !yaml.contains("resolvedCommit"),
            "resolvedCommit must not be serialized; got: {yaml}"
        );
    }

    #[test]
    fn serialize_omits_resolved_attr() {
        let pinned = PinnedPackageEntry {
            name: "go".parse().unwrap(),
            version: "1.21.13".parse().unwrap(),
            resolved_commit: None,
            resolved_attr: Some("go_1_21".to_string()),
        };

        let yaml = serde_yaml::to_string(&pinned).unwrap();

        assert!(
            !yaml.contains("resolvedAttr"),
            "resolvedAttr must not be serialized; got: {yaml}"
        );
    }

    #[test]
    fn serialize_emits_name_and_version() {
        let pinned = PinnedPackageEntry {
            name: "go".parse().unwrap(),
            version: "1.21.13".parse().unwrap(),
            resolved_commit: Some("5ed6275".to_string()),
            resolved_attr: Some("go_1_21".to_string()),
        };

        let yaml = serde_yaml::to_string(&pinned).unwrap();

        assert!(yaml.contains("name: go"), "missing name; got: {yaml}");
        assert!(
            yaml.contains("version: 1.21.13"),
            "missing version; got: {yaml}"
        );
    }

    #[test]
    fn round_trip_drops_resolved_fields() {
        let legacy_yaml = r#"
name: go
version: "1.21.13"
resolvedCommit: "5ed6275"
resolvedAttr: "go_1_21"
"#;

        let first: PinnedPackageEntry = serde_yaml::from_str(legacy_yaml).unwrap();
        let re_serialized = serde_yaml::to_string(&first).unwrap();
        let second: PinnedPackageEntry = serde_yaml::from_str(&re_serialized).unwrap();

        assert!(second.resolved_commit.is_none());
        assert!(second.resolved_attr.is_none());
    }

    #[test]
    fn deserialize_still_accepts_legacy_resolved_fields() {
        let legacy_yaml = r#"
name: go
version: "1.21.13"
resolvedCommit: "5ed6275"
resolvedAttr: "go_1_21"
"#;

        let pinned: PinnedPackageEntry = serde_yaml::from_str(legacy_yaml).unwrap();

        assert_eq!(pinned.name.as_str(), "go");
        assert_eq!(pinned.version.as_str(), "1.21.13");
        assert_eq!(pinned.resolved_commit.as_deref(), Some("5ed6275"));
        assert_eq!(pinned.resolved_attr.as_deref(), Some("go_1_21"));
    }
}
