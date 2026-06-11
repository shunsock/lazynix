use serde::{Deserialize, Serialize};

use crate::error::{LazyNixError, Result};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct Settings {
    pub override_stable_package: Option<String>,
}

pub fn validate_registry_url(url: &str) -> Result<()> {
    // Empty URL is invalid
    if url.is_empty() {
        return Err(LazyNixError::SettingsValidation(
            "Registry URL cannot be empty".to_string(),
        ));
    }

    // Must start with "github:"
    if !url.starts_with("github:") {
        return Err(LazyNixError::SettingsValidation(format!(
            "Invalid registry URL format: '{}'. Must start with 'github:' (e.g., 'github:NixOS/nixpkgs/nixos-25.06')",
            url
        )));
    }

    // Parse the rest after "github:"
    let rest = &url[7..]; // Skip "github:"
    let parts: Vec<&str> = rest.split('/').collect();

    // Must have exactly 3 parts: OWNER/REPO/BRANCH
    if parts.len() != 3 {
        return Err(LazyNixError::SettingsValidation(format!(
            "Invalid registry URL format: '{}'. Expected format: 'github:OWNER/REPO/BRANCH' (e.g., 'github:NixOS/nixpkgs/nixos-25.06')",
            url
        )));
    }

    // Validate each part is non-empty and contains valid characters
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            let part_name = match i {
                0 => "OWNER",
                1 => "REPO",
                2 => "BRANCH",
                _ => "part",
            };
            return Err(LazyNixError::SettingsValidation(format!(
                "Invalid registry URL: {} cannot be empty in '{}'",
                part_name, url
            )));
        }

        // Allow alphanumeric, hyphens, underscores, and dots
        if !part
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
        {
            return Err(LazyNixError::SettingsValidation(format!(
                "Invalid registry URL: '{}' contains invalid characters. Only alphanumeric, hyphens, underscores, and dots are allowed",
                url
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_deserialize_valid_yaml() {
        let yaml = r#"
override-stable-package: "github:NixOS/nixpkgs/nixos-25.06"
"#;
        let settings: Settings = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(
            settings.override_stable_package,
            Some("github:NixOS/nixpkgs/nixos-25.06".to_string())
        );
    }

    #[test]
    fn test_settings_deserialize_no_override() {
        let yaml = r#"
# No override specified
"#;
        let settings: Settings = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(settings.override_stable_package, None);
    }

    #[test]
    fn test_settings_deserialize_empty_yaml() {
        let yaml = "";
        let settings: Settings = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(settings.override_stable_package, None);
    }

    #[test]
    fn test_validate_registry_url_valid() {
        // Standard nixpkgs URLs
        assert!(validate_registry_url("github:NixOS/nixpkgs/nixos-25.06").is_ok());
        assert!(validate_registry_url("github:NixOS/nixpkgs/nixos-25.11").is_ok());
        assert!(validate_registry_url("github:NixOS/nixpkgs/nixos-unstable").is_ok());

        // Custom fork
        assert!(validate_registry_url("github:myuser/nixpkgs/custom-branch").is_ok());

        // With underscores and dots
        assert!(validate_registry_url("github:my_user/nix_pkgs/branch.name").is_ok());
    }

    #[test]
    fn test_validate_registry_url_invalid() {
        // Empty URL
        assert!(validate_registry_url("").is_err());

        // Wrong prefix
        assert!(validate_registry_url("https://github.com/NixOS/nixpkgs").is_err());
        assert!(validate_registry_url("gitlab:NixOS/nixpkgs/main").is_err());

        // Missing parts
        assert!(validate_registry_url("github:NixOS/nixpkgs").is_err()); // Missing branch
        assert!(validate_registry_url("github:NixOS").is_err()); // Missing repo and branch
        assert!(validate_registry_url("github:").is_err()); // Missing everything

        // Too many parts
        assert!(validate_registry_url("github:NixOS/nixpkgs/branch/extra").is_err());

        // Empty parts
        assert!(validate_registry_url("github://nixpkgs/branch").is_err()); // Empty owner
        assert!(validate_registry_url("github:NixOS//branch").is_err()); // Empty repo
        assert!(validate_registry_url("github:NixOS/nixpkgs/").is_err()); // Empty branch

        // Invalid characters
        assert!(validate_registry_url("github:NixOS/nixpkgs/branch@tag").is_err());
        assert!(validate_registry_url("github:Nix OS/nixpkgs/branch").is_err()); // space
        assert!(validate_registry_url("github:NixOS/nix$pkgs/branch").is_err()); // $
    }
}
