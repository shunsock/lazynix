use crate::values::RegistryUrl;
use serde::{Deserialize, Serialize};

/// Optional per-user settings from `lazynix-settings.yaml`.
///
/// URL validation is handled by [`RegistryUrl`] during deserialization,
/// so a successfully parsed `Settings` is always valid.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct Settings {
    pub override_stable_package: Option<RegistryUrl>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_override_url() {
        let yaml = r#"
override-stable-package: "github:NixOS/nixpkgs/nixos-26.05"
"#;

        let settings: Settings = serde_yaml::from_str(yaml).unwrap();

        assert_eq!(
            settings.override_stable_package,
            Some("github:NixOS/nixpkgs/nixos-26.05".parse().unwrap())
        );
    }

    #[test]
    fn deserializes_missing_override_as_none() {
        let yaml = "# No override specified\n";

        let settings: Settings = serde_yaml::from_str(yaml).unwrap();

        assert_eq!(settings.override_stable_package, None);
    }

    #[test]
    fn rejects_invalid_override_url_at_parse_time() {
        let yaml = r#"
override-stable-package: "https://github.com/NixOS/nixpkgs"
"#;

        let result = serde_yaml::from_str::<Settings>(yaml);

        let message = result.unwrap_err().to_string();
        assert!(message.contains("Invalid registry URL"), "got: {message}");
    }
}
