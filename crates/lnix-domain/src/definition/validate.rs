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

// CONSTRAINT: 完全一致と接頭辞衝突の両方を拒否しなければならない。
// REASON: reader は名前で行照合するため、どちらも attr を誤割当させる。
// NOTE: sorting makes one adjacent-pairs scan enough for both,
// since '-'/'.' sort smallest among the allowed bytes.
fn reject_colliding_pinned_names(
    pinned: &[crate::PinnedPackageEntry],
    encode: impl Fn(&crate::PackageName, &crate::PackageVersion) -> String,
    make_error: impl Fn(PinnedNameCollision) -> ValidationError,
) -> Result<(), ValidationError> {
    let mut encoded: Vec<(&crate::PinnedPackageEntry, String)> = pinned
        .iter()
        .map(|entry| (entry, encode(&entry.name, &entry.version)))
        .collect();
    encoded.sort_by(|a, b| a.1.cmp(&b.1));

    for pair in encoded.windows(2) {
        let (first, first_encoded) = &pair[0];
        let (second, second_encoded) = &pair[1];
        let Some(collided) = colliding_name(first_encoded, second_encoded) else {
            continue;
        };
        return Err(make_error(PinnedNameCollision {
            first_name: first.name.as_str().to_string(),
            first_version: first.version.as_str().to_string(),
            second_name: second.name.as_str().to_string(),
            second_version: second.version.as_str().to_string(),
            encoded: collided,
        }));
    }
    Ok(())
}

/// Returns a description when `a` and `b` collide: either an exact
/// match, or the shorter being a proper prefix of the longer that is
/// immediately followed by `.` or `-`. For a prefix match, the result
/// is `"{shorter}+{rest}"`, so `longer == shorter` followed by `rest`.
/// Returns `None` when they neither match nor overlap this way.
fn colliding_name(a: &str, b: &str) -> Option<String> {
    if a == b {
        return Some(a.to_string());
    }
    let (shorter, longer) = if a.len() < b.len() { (a, b) } else { (b, a) };
    let boundary = longer.as_bytes().get(shorter.len()).copied()?;
    let collides = longer.starts_with(shorter) && matches!(boundary, b'.' | b'-');
    collides.then(|| format!("{shorter}+{}", &longer[shorter.len()..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(yaml: &str) -> DevShellDefinition {
        serde_yaml::from_str(yaml).unwrap()
    }

    // NOTE: every test below only checks the Ok/Err shape, so these
    // two helpers carry that one shared assertion.
    fn assert_valid(config: &DevShellDefinition) {
        assert_eq!(validate_config(config).unwrap(), Vec::<Diagnostic>::new());
    }

    fn assert_rejects(config: &DevShellDefinition, expected: ValidationError) {
        assert_eq!(validate_config(config).unwrap_err(), expected);
    }

    #[test]
    fn accepts_config_with_valid_task() {
        let yaml = r#"devShell: {package: {stable: [{name: bash}]}, task: {sync: {commands: ["uv sync"]}}}"#;
        assert_valid(&parse(yaml));
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

    fn parse_pinned(entries: &[(&str, &str)]) -> DevShellDefinition {
        parse(&pinned_yaml(entries))
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
            let config = parse_pinned(&[(name, version)]);
            assert_rejects(&config, invalid_version(name, version));
        }
    }

    #[test]
    fn rejects_colliding_pinned_entries() {
        let cases = [
            ("go", "1.2.3", "go", "1-2-3", "nixpkgs--go--1-2-3"),
            ("go", "1.21.13", "go", "1.21.13", "nixpkgs--go--1-21-13"),
        ];
        for (n1, v1, n2, v2, enc) in cases {
            let config = parse_pinned(&[(n1, v1), (n2, v2)]);
            let err = ValidationError::PinnedInputNameCollision(collision(n1, v1, n2, v2, enc));
            assert_rejects(&config, err);
        }
    }

    // NOTE: exact-match and prefix collisions both raise
    // PinnedBindingNameCollision, so one table covers both.
    #[test]
    fn rejects_colliding_pinned_binding_names() {
        let cases = [
            ("go", "1.21.13", "go-1", "21.13", "pinnedPkgs-go-1-21-13"),
            ("a", "b", "a-b.x", "0", "pinnedPkgs-a-b+.x-0"),
        ];
        for (n1, v1, n2, v2, enc) in cases {
            let config = parse_pinned(&[(n1, v1), (n2, v2)]);
            let err = ValidationError::PinnedBindingNameCollision(collision(n1, v1, n2, v2, enc));
            assert_rejects(&config, err);
        }
    }

    #[test]
    fn accepts_valid_pinned_configs() {
        assert_valid(&parse_pinned(&[(
            "dotnet-sdk_11",
            "11.0.100-preview.7.26381.103",
        )]));
        assert_valid(&parse_pinned(&[("go", "1.21.13"), ("python", "3.12.0")]));
        // NOTE: "go-1-1" prefixes "go-1-10", but the next byte is
        // '0', not '.'/'-', so this is not a collision.
        assert_valid(&parse_pinned(&[("go", "1.1"), ("go", "1.10")]));
    }

    #[test]
    fn rejects_task_with_empty_commands() {
        let yaml =
            r#"devShell: {package: {stable: [{name: bash}]}, task: {empty-task: {commands: []}}}"#;
        let config = parse(yaml);
        let expected = ValidationError::EmptyTaskCommands("empty-task".to_string());
        assert_rejects(&config, expected);
    }
}
