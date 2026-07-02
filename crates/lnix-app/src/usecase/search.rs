//! `lnix search` — discover available package versions via nix-versions.

use lnix_domain::PackageName;

use crate::deps::Deps;
use crate::error::ApplicationError;

/// Searches for versions of `package_name`, optionally constrained by
/// `version`, and passes the registry output straight through.
pub fn search(
    d: &Deps,
    package_name: &str,
    version: Option<&str>,
    json: bool,
    one: bool,
) -> Result<i32, ApplicationError> {
    let package_name: PackageName = package_name.parse()?;

    let output = d.resolver.search(&package_name, version, json, one)?;
    d.out.info(output.trim_end());
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mocks::*;

    #[test]
    fn passes_registry_output_through() {
        // Arrange
        let m = Mocks::with_missing_config();

        // Act
        let code = search(&m.deps(), "go", Some(">=1.20"), false, true).unwrap();

        // Assert
        assert_eq!(code, 0);
        assert!(m.out.infos().iter().any(|line| line.contains("go 1.21.13")));
    }

    #[test]
    fn rejects_invalid_package_name_before_spawning() {
        // Arrange
        let m = Mocks::with_missing_config();

        // Act
        let result = search(&m.deps(), "pkg; rm -rf /", None, false, false);

        // Assert
        assert!(matches!(result, Err(ApplicationError::InvalidInput(_))));
    }
}
