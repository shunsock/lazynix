/// Generate a minimal valid lazynix.yaml config
#[allow(dead_code)]
pub fn minimal_config() -> String {
    r#"devShell:
  allowUnfree: false
  package:
    stable:
      - bash
    unstable: []
  shellHook: []
"#
    .to_string()
}

/// Generate a config with specified packages
pub fn config_with_packages(stable: &[&str], unstable: &[&str]) -> String {
    format!(
        r#"devShell:
  allowUnfree: false
  package:
    stable:
{}
    unstable:
{}
  shellHook: []
"#,
        if stable.is_empty() {
            "      []".to_string()
        } else {
            stable
                .iter()
                .map(|p| format!("      - {}", p))
                .collect::<Vec<_>>()
                .join("\n")
        },
        if unstable.is_empty() {
            "      []".to_string()
        } else {
            unstable
                .iter()
                .map(|p| format!("      - {}", p))
                .collect::<Vec<_>>()
                .join("\n")
        }
    )
}

/// Generate a config with test commands
#[allow(dead_code)]
pub fn config_with_test_commands(tests: &[&str]) -> String {
    format!(
        r#"devShell:
  allowUnfree: false
  package:
    stable:
      - bash
  test:
{}
"#,
        tests
            .iter()
            .map(|t| format!("    - {}", t))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

/// Generate a config with task definitions
#[allow(dead_code)]
pub fn config_with_tasks(tasks: &[(&str, Vec<&str>, Option<&str>)]) -> String {
    let mut task_defs = String::new();
    for (name, commands, description) in tasks {
        task_defs.push_str(&format!("    {}:\n", name));
        if let Some(desc) = description {
            task_defs.push_str(&format!("      description: \"{}\"\n", desc));
        }
        task_defs.push_str("      commands:\n");
        for cmd in commands {
            task_defs.push_str(&format!("        - {}\n", cmd));
        }
    }

    format!(
        r#"devShell:
  allowUnfree: false
  package:
    stable:
      - bash
  task:
{}
"#,
        task_defs
    )
}

/// Generate invalid YAML for error testing
#[allow(dead_code)]
pub fn invalid_yaml_config() -> String {
    r#"devShell:
  allowUnfree: false
  package:
    stable:
      - hello
    - this is invalid yaml syntax
"#
    .to_string()
}

/// Generate a config with invalid package names
#[allow(dead_code)]
pub fn config_with_invalid_package_names() -> String {
    config_with_packages(&["nonexistent-pkg-xyz-12345"], &[])
}
