//! Composes the dev shell's `shellHook` from its ordered fragments.
//!
//! Order is significant: dotenv files load first, then explicit env
//! vars, then shell aliases, then the user's own hook lines, and
//! finally the test-runner block (when in test mode).

use crate::{DevShellDefinition, EnvVar};

use super::path::resolve_path;
use super::test_runner::render_test_execution;

fn render_dotenv_loading(dotenv_files: &[String]) -> String {
    dotenv_files
        .iter()
        .map(|file| {
            let resolved_path = resolve_path(file);
            format!(
                r#"            # Load dotenv file: {}
            set -a
            source "{}"
            set +a"#,
                file, resolved_path
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn render_envvar_export(envvars: &[EnvVar]) -> String {
    envvars
        .iter()
        .map(|envvar| format!("            export {}={}", envvar.name, envvar.value))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_shell_alias_loading(alias_files: &[String]) -> String {
    alias_files
        .iter()
        .map(|file| {
            let resolved_path = resolve_path(file);
            format!(
                r#"            # Load shell aliases: {}
            if [ -f "{}" ]; then
                eval "$(grep '^alias ' "{}" 2>/dev/null || true)"
            fi"#,
                file, resolved_path, resolved_path
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn render_user_hook(shell_hook: &[String]) -> String {
    if shell_hook.is_empty() {
        return String::from("            echo \"Welcome to LazyNix DevShell!\"");
    }
    shell_hook
        .iter()
        .map(|cmd| format!("            {}", cmd))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_test_block(tests: &[String]) -> String {
    let test_execution = render_test_execution(tests);
    if test_execution.is_empty() {
        return String::new();
    }
    format!(
        r#"if [ "$LAZYNIX_TEST_MODE" = "1" ]; then
{}
fi"#,
        test_execution
    )
}

/// Assembles the full `shellHook` body in fragment order.
pub(super) fn compose_shell_hook(config: &DevShellDefinition) -> String {
    let dev_shell = &config.dev_shell;
    let env = dev_shell.env.as_ref();

    let fragments = [
        env.map(|e| render_dotenv_loading(&e.dotenv))
            .unwrap_or_default(),
        env.map(|e| render_envvar_export(&e.envvar))
            .unwrap_or_default(),
        render_shell_alias_loading(&dev_shell.shell_alias),
        render_user_hook(&dev_shell.shell_hook),
        render_test_block(&dev_shell.test),
    ];

    fragments
        .into_iter()
        .filter(|fragment| !fragment.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config_from_yaml(yaml: &str) -> DevShellDefinition {
        serde_yaml::from_str(yaml).unwrap()
    }

    #[test]
    fn defaults_to_welcome_message_when_empty() {
        // Arrange
        let config = config_from_yaml("devShell:\n  package:\n    stable:\n      - name: bash\n");

        // Act
        let hook = compose_shell_hook(&config);

        // Assert
        assert_eq!(hook, "            echo \"Welcome to LazyNix DevShell!\"");
    }

    #[test]
    fn orders_dotenv_then_envvar_then_alias_then_user_hook() {
        // Arrange
        let config = config_from_yaml(
            r#"
devShell:
  package:
    stable:
      - name: bash
  shellHook:
    - echo 'user hook'
  shellAlias:
    - ./aliases.sh
  env:
    dotenv:
      - .env
    envvar:
      - name: TEST
        value: value
"#,
        );

        // Act
        let hook = compose_shell_hook(&config);

        // Assert
        let dotenv_pos = hook.find("Load dotenv").unwrap();
        let envvar_pos = hook.find("export TEST=").unwrap();
        let alias_pos = hook.find("Load shell aliases").unwrap();
        let user_pos = hook.find("echo 'user hook'").unwrap();
        assert!(dotenv_pos < envvar_pos);
        assert!(envvar_pos < alias_pos);
        assert!(alias_pos < user_pos);
    }

    #[test]
    fn gates_test_block_behind_test_mode() {
        // Arrange
        let config = config_from_yaml(
            r#"
devShell:
  package:
    stable:
      - name: bash
  test:
    - pytest
"#,
        );

        // Act
        let hook = compose_shell_hook(&config);

        // Assert
        assert!(hook.contains("if [ \"$LAZYNIX_TEST_MODE\" = \"1\" ]"));
        assert!(hook.contains("pytest"));
    }
}
