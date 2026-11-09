use std::collections::HashMap;

use crate::definition::dev_shell::DevShellDefinition;
use crate::error::{Diagnostic, PinnedNameCollision, ValidationError};
use crate::service::flake::pinned::{binding_name, input_name};

/// Checks cross-field constraints that value objects cannot express.
///
/// Field-level invariants (package names, task names, env var names,
/// versions) are already guaranteed by the type system at this point.
///
/// Returns non-fatal findings as [`Diagnostic`] values instead of
/// printing them, so this function stays free of I/O; the caller
/// decides how to display them.
pub fn validate_config(config: &DevShellDefinition) -> Result<Vec<Diagnostic>, ValidationError> {
    if let Some(tasks) = &config.dev_shell.task {
        for (task_name, task_def) in tasks {
            if task_def.commands.is_empty() {
                return Err(ValidationError::EmptyTaskCommands(
                    task_name.as_str().to_string(),
                ));
            }
        }
    }

    for entry in &config.dev_shell.package.pinned {
        if !entry.version.as_str().chars().all(is_allowed_version_char) {
            return Err(ValidationError::InvalidPinnedVersionCharacters {
                name: entry.name.as_str().to_string(),
                version: entry.version.as_str().to_string(),
            });
        }
    }

    reject_colliding_pinned_names(&config.dev_shell.package.pinned, input_name, |c| {
        ValidationError::PinnedInputNameCollision(Box::new(c))
    })?;
    reject_colliding_pinned_names(&config.dev_shell.package.pinned, binding_name, |c| {
        ValidationError::PinnedBindingNameCollision(Box::new(c))
    })?;

    let mut diagnostics = Vec::new();
    let package = &config.dev_shell.package;
    let has_no_packages =
        package.stable.is_empty() && package.unstable.is_empty() && package.pinned.is_empty();
    if has_no_packages {
        diagnostics.push(Diagnostic::NoPackages);
    }

    Ok(diagnostics)
}

// CONSTRAINT: charset はエラー文言と同じ集合でなければならない。
// REASON: dot-to-hyphen 後も有効な Nix 識別子でなければならない。
fn is_allowed_version_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-')
}

// CONSTRAINT: 両エンコードの衝突を拒否しなければならない。
// REASON: binding は input より区切り文字が少なく別に衝突しうる。
fn reject_colliding_pinned_names(
    pinned: &[crate::PinnedPackageEntry],
    encode: impl Fn(&crate::PackageName, &crate::PackageVersion) -> String,
    make_error: impl Fn(PinnedNameCollision) -> ValidationError,
) -> Result<(), ValidationError> {
    let mut seen: HashMap<String, &crate::PinnedPackageEntry> = HashMap::new();
    for entry in pinned {
        let encoded = encode(&entry.name, &entry.version);
        if let Some(first) = seen.get(&encoded) {
            return Err(make_error(PinnedNameCollision {
                first_name: first.name.as_str().to_string(),
                first_version: first.version.as_str().to_string(),
                second_name: entry.name.as_str().to_string(),
                second_version: entry.version.as_str().to_string(),
                encoded,
            }));
        }
        seen.insert(encoded, entry);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(yaml: &str) -> DevShellDefinition {
        serde_yaml::from_str(yaml).unwrap()
    }

    #[test]
    fn accepts_config_with_valid_task() {
        let yaml = r#"devShell: {package: {stable: [{name: bash}]}, task: {sync: {commands: ["uv sync"]}}}"#;
        let config = parse(yaml);

        assert_eq!(validate_config(&config).unwrap(), Vec::<Diagnostic>::new());
    }

    #[test]
    fn reports_no_packages_as_diagnostic() {
        let config = parse(r#"devShell: {package: {stable: []}}"#);

        assert_eq!(
            validate_config(&config).unwrap(),
            vec![Diagnostic::NoPackages]
        );
    }

    fn escape_yaml(v: &str) -> String {
        v.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
    }

    fn pinned_yaml(entries: &[(&str, &str)]) -> String {
        let items = entries
            .iter()
            .map(|(n, v)| format!(r#"{{name: {n}, version: "{}"}}"#, escape_yaml(v)))
            .collect::<Vec<_>>()
            .join(", ");
        format!("devShell: {{package: {{pinned: [{items}]}}}}")
    }

    fn invalid_version(name: &str, version: &str) -> ValidationError {
        ValidationError::InvalidPinnedVersionCharacters {
            name: name.into(),
            version: version.into(),
        }
    }

    fn collision(
        fname: &str,
        fver: &str,
        sname: &str,
        sver: &str,
        encoded: &str,
    ) -> Box<PinnedNameCollision> {
        Box::new(PinnedNameCollision {
            first_name: fname.into(),
            first_version: fver.into(),
            second_name: sname.into(),
            second_version: sver.into(),
            encoded: encoded.into(),
        })
    }

    #[test]
    fn rejects_pinned_version_with_invalid_characters() {
        let cases = [("go", ">=1.20,<1.22"), ("go", "1.0\n\"evil\" = true;")];
        for (name, version) in cases {
            let config = parse(&pinned_yaml(&[(name, version)]));
            assert_eq!(
                validate_config(&config).unwrap_err(),
                invalid_version(name, version)
            );
        }
    }

    #[test]
    fn rejects_colliding_pinned_entries() {
        let cases = [
            ("go", "1.2.3", "go", "1-2-3", "nixpkgs--go--1-2-3"),
            ("go", "1.21.13", "go", "1.21.13", "nixpkgs--go--1-21-13"),
        ];
        for (n1, v1, n2, v2, encoded) in cases {
            let config = parse(&pinned_yaml(&[(n1, v1), (n2, v2)]));
            assert_eq!(
                validate_config(&config).unwrap_err(),
                ValidationError::PinnedInputNameCollision(collision(n1, v1, n2, v2, encoded))
            );
        }
    }

    #[test]
    fn rejects_colliding_pinned_binding_names() {
        let (n1, v1, n2, v2) = ("go", "1.21.13", "go-1", "21.13");
        let config = parse(&pinned_yaml(&[(n1, v1), (n2, v2)]));

        assert_eq!(
            validate_config(&config).unwrap_err(),
            ValidationError::PinnedBindingNameCollision(collision(
                n1,
                v1,
                n2,
                v2,
                "pinnedPkgs-go-1-21-13"
            ))
        );
    }

    #[test]
    fn accepts_valid_pinned_configs() {
        let single = parse(&pinned_yaml(&[(
            "dotnet-sdk_11",
            "11.0.100-preview.7.26381.103",
        )]));
        let multiple = parse(&pinned_yaml(&[("go", "1.21.13"), ("python", "3.12.0")]));

        assert_eq!(validate_config(&single).unwrap(), Vec::<Diagnostic>::new());
        assert_eq!(
            validate_config(&multiple).unwrap(),
            Vec::<Diagnostic>::new()
        );
    }

    #[test]
    fn rejects_task_with_empty_commands() {
        let yaml =
            r#"devShell: {package: {stable: [{name: bash}]}, task: {empty-task: {commands: []}}}"#;
        let config = parse(yaml);

        assert_eq!(
            validate_config(&config).unwrap_err(),
            ValidationError::EmptyTaskCommands("empty-task".to_string())
        );
    }
}
