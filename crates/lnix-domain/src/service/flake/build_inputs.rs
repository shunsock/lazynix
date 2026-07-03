//! Renders the `buildInputs` list grouped by package source.
//!
//! Stable and unstable packages reference their respective nixpkgs
//! imports; pinned packages reference their per-package imports. Empty
//! groups are omitted, and an all-empty list yields a placeholder
//! comment so the generated flake stays valid.

use crate::{DevShellDefinition, PackageEntry, PinnedPackageEntry};

use super::pinned;

fn render_channel(import_name: &str, entries: &[PackageEntry]) -> String {
    entries
        .iter()
        .map(|entry| format!("            {}.{}", import_name, entry.name))
        .collect::<Vec<_>>()
        .join("\n")
}

fn labeled_group(label: &str, body: &str) -> Option<String> {
    if body.is_empty() {
        return None;
    }
    Some(format!("            # {} packages\n{}", label, body))
}

/// Assembles the full `buildInputs` body from all three package sources.
pub(super) fn render_build_inputs(
    config: &DevShellDefinition,
    resolved: &[&PinnedPackageEntry],
) -> String {
    let package = &config.dev_shell.package;
    let groups = [
        labeled_group("Stable", &render_channel("stablePackages", &package.stable)),
        labeled_group(
            "Unstable",
            &render_channel("unstablePackages", &package.unstable),
        ),
        labeled_group("Pinned", &pinned::render_packages(resolved)),
    ];

    let parts: Vec<String> = groups.into_iter().flatten().collect();
    if parts.is_empty() {
        return String::from("            # No packages specified");
    }
    parts.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config_from_yaml(yaml: &str) -> DevShellDefinition {
        serde_yaml::from_str(yaml).unwrap()
    }

    #[test]
    fn emits_placeholder_when_no_packages() {
        // Arrange
        let config = config_from_yaml("devShell:\n  package: {}\n");

        // Act
        let build_inputs = render_build_inputs(&config, &[]);

        // Assert
        assert_eq!(build_inputs, "            # No packages specified");
    }

    #[test]
    fn groups_stable_and_unstable_packages() {
        // Arrange
        let config = config_from_yaml(
            r#"
devShell:
  package:
    stable:
      - name: python312
      - name: gcc
    unstable:
      - name: rust-analyzer
"#,
        );

        // Act
        let build_inputs = render_build_inputs(&config, &[]);

        // Assert
        assert!(build_inputs.contains("# Stable packages"));
        assert!(build_inputs.contains("stablePackages.python312"));
        assert!(build_inputs.contains("stablePackages.gcc"));
        assert!(build_inputs.contains("# Unstable packages"));
        assert!(build_inputs.contains("unstablePackages.rust-analyzer"));
    }
}
