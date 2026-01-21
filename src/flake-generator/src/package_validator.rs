pub fn is_valid_package_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    // Reject dots at start or end
    if name.starts_with('.') || name.ends_with('.') {
        return false;
    }

    // Reject consecutive dots
    if name.contains("..") {
        return false;
    }

    // Allow alphanumeric, hyphens, underscores, and dots
    name.chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_package_names() {
        assert!(is_valid_package_name("python312"));
        assert!(is_valid_package_name("rust-analyzer"));
        assert!(is_valid_package_name("node_js"));
        assert!(is_valid_package_name("gcc"));

        // Nested attributes (Issue #11 cases)
        assert!(is_valid_package_name("python312Packages.pip"));
        assert!(is_valid_package_name("python312Packages.virtualenv"));
        assert!(is_valid_package_name("nodePackages.typescript"));
        assert!(is_valid_package_name("lib.strings.concatStringsSep"));
    }

    #[test]
    fn test_invalid_package_names() {
        assert!(!is_valid_package_name(""));
        assert!(!is_valid_package_name("pkg@version"));
        assert!(!is_valid_package_name("pkg with space"));
        assert!(!is_valid_package_name("pkg/path"));

        // Edge cases with dots
        assert!(!is_valid_package_name(".pkg")); // starts with dot
        assert!(!is_valid_package_name("pkg.")); // ends with dot
        assert!(!is_valid_package_name("pkg..name")); // double dots

        // Shell metacharacters (security)
        assert!(!is_valid_package_name("pkg$var"));
        assert!(!is_valid_package_name("pkg;cmd"));
        assert!(!is_valid_package_name("pkg|cmd"));
    }
}
