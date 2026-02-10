use crate::config::{Config, EnvVar};

fn is_absolute_path(path: &str) -> bool {
    path.starts_with('/')
}

fn resolve_path(path: &str) -> String {
    if path.starts_with("~/") {
        // Tilde expansion at runtime
        format!("$HOME/{}", &path[2..])
    } else if is_absolute_path(path) {
        path.to_string()
    } else {
        // For relative paths, resolve them relative to $PWD at runtime
        format!("$PWD/{}", path.trim_start_matches("./"))
    }
}

fn render_envvar_export(envvars: &[EnvVar]) -> String {
    if envvars.is_empty() {
        return String::new();
    }

    envvars
        .iter()
        .map(|envvar| format!("            export {}={}", envvar.name, envvar.value))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_dotenv_loading(dotenv_files: &[String]) -> String {
    if dotenv_files.is_empty() {
        return String::new();
    }

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

fn render_shell_alias_loading(alias_files: &[String]) -> String {
    if alias_files.is_empty() {
        return String::new();
    }

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

fn render_test_execution(tests: &[String]) -> String {
    if tests.is_empty() {
        return String::new();
    }

    let test_commands = tests
        .iter()
        .map(|cmd| format!("        \"{}\"", cmd))
        .collect::<Vec<_>>()
        .join(" \\\n");

    format!(
        r#"TESTS_FAILED=0
TESTS_PASSED=0

echo "Running tests..."
echo "================="

for test_cmd in \
{}
do
    echo ""
    echo "Running: $test_cmd"
    echo "---"
    if bash -c "$test_cmd"; then
        TESTS_PASSED=$((TESTS_PASSED + 1))
        echo "[PASS] $test_cmd"
    else
        TESTS_FAILED=$((TESTS_FAILED + 1))
        echo "[FAIL] $test_cmd"
    fi
done

echo ""
echo "================="
echo "Test Results: $TESTS_PASSED passed, $TESTS_FAILED failed"

if [ $TESTS_FAILED -gt 0 ]; then
    exit 1
fi"#,
        test_commands
    )
}

pub fn render_flake(config: &Config, override_stable_package: Option<&str>) -> String {
    let allow_unfree = config.dev_shell.allow_unfree;

    // Determine stable nixpkgs URL (use override if provided, otherwise default)
    let stable_url = override_stable_package.unwrap_or("github:NixOS/nixpkgs/nixos-25.11");

    // Format stable packages
    let stable_packages = if config.dev_shell.package.stable.is_empty() {
        String::new()
    } else {
        config
            .dev_shell
            .package
            .stable
            .iter()
            .map(|pkg| format!("            stablePackages.{}", pkg))
            .collect::<Vec<_>>()
            .join("\n")
    };

    // Format unstable packages
    let unstable_packages = if config.dev_shell.package.unstable.is_empty() {
        String::new()
    } else {
        config
            .dev_shell
            .package
            .unstable
            .iter()
            .map(|pkg| format!("            unstablePackages.{}", pkg))
            .collect::<Vec<_>>()
            .join("\n")
    };

    // Combine package lists
    let build_inputs = match (!stable_packages.is_empty(), !unstable_packages.is_empty()) {
        (true, true) => format!(
            "            # Stable packages\n{}\n            # Unstable packages\n{}",
            stable_packages, unstable_packages
        ),
        (true, false) => format!("            # Stable packages\n{}", stable_packages),
        (false, true) => format!("            # Unstable packages\n{}", unstable_packages),
        (false, false) => String::from("            # No packages specified"),
    };

    // Format dotenv loading
    let dotenv_hook = if let Some(env) = &config.dev_shell.env {
        render_dotenv_loading(&env.dotenv)
    } else {
        String::new()
    };

    // Format envvar export
    let envvar_hook = if let Some(env) = &config.dev_shell.env {
        render_envvar_export(&env.envvar)
    } else {
        String::new()
    };

    // Format shell alias loading
    let shell_alias_hook = render_shell_alias_loading(&config.dev_shell.shell_alias);

    // Format user-defined shell hooks
    let user_shell_hook = if config.dev_shell.shell_hook.is_empty() {
        String::from("            echo \"Welcome to LazyNix DevShell!\"")
    } else {
        config
            .dev_shell
            .shell_hook
            .iter()
            .map(|cmd| format!("            {}", cmd))
            .collect::<Vec<_>>()
            .join("\n")
    };

    // Generate test execution logic
    let test_execution = if !config.dev_shell.test.is_empty() {
        render_test_execution(&config.dev_shell.test)
    } else {
        String::new()
    };

    // Combine dotenv loading, envvar export, shell alias loading, and user shell hooks
    let mut shell_hook_parts = Vec::new();
    if !dotenv_hook.is_empty() {
        shell_hook_parts.push(dotenv_hook);
    }
    if !envvar_hook.is_empty() {
        shell_hook_parts.push(envvar_hook);
    }
    if !shell_alias_hook.is_empty() {
        shell_hook_parts.push(shell_alias_hook);
    }
    shell_hook_parts.push(user_shell_hook);

    // Add test execution logic if tests are defined
    if !test_execution.is_empty() {
        let test_block = format!(
            r#"if [ "$LAZYNIX_TEST_MODE" = "1" ]; then
{}
fi"#,
            test_execution
        );
        shell_hook_parts.push(test_block);
    }

    let shell_hook = shell_hook_parts.join("\n\n");

    format!(
        r#"# Generated by LazyNix - DO NOT EDIT MANUALLY
# This file is automatically generated from lazynix.yaml
# To make changes, edit lazynix.yaml and run: lnix develop

{{
  description = "DevShell generated by LazyNix";

  inputs = {{
    nixpkgs.url = "{}";
    nixpkgs-unstable.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  }};

  outputs = {{ self, nixpkgs, nixpkgs-unstable, flake-utils }}:
    flake-utils.lib.eachDefaultSystem (system:
      let
        stablePackages = import nixpkgs {{
          inherit system;
          config.allowUnfree = {};
        }};
        unstablePackages = import nixpkgs-unstable {{
          inherit system;
          config.allowUnfree = {};
        }};
      in
      {{
        devShells.default = stablePackages.mkShell {{
          buildInputs = [
{}
          ];

          shellHook = ''
{}
          '';
        }};
      }}
    );
}}
"#,
        stable_url, allow_unfree, allow_unfree, build_inputs, shell_hook
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, DevShell, Env, EnvVar, Package};

    // Test fixtures and helpers

    fn create_default_config() -> Config {
        Config {
            dev_shell: DevShell {
                allow_unfree: false,
                package: Package {
                    stable: vec!["bash".to_string()],
                    unstable: vec![],
                },
                shell_hook: vec![],
                env: None,
                test: vec![],
                task: None,
                shell_alias: vec![],
            },
        }
    }

    fn create_config_with_env(env: Env) -> Config {
        let mut config = create_default_config();
        config.dev_shell.env = Some(env);
        config
    }

    fn create_config_with_packages(stable: Vec<String>, unstable: Vec<String>) -> Config {
        let mut config = create_default_config();
        config.dev_shell.package.stable = stable;
        config.dev_shell.package.unstable = unstable;
        config
    }

    #[allow(dead_code)]
    fn create_config_with_shell_hook(shell_hook: Vec<String>) -> Config {
        let mut config = create_default_config();
        config.dev_shell.shell_hook = shell_hook;
        config
    }

    fn create_config_with_allow_unfree(allow_unfree: bool) -> Config {
        let mut config = create_default_config();
        config.dev_shell.allow_unfree = allow_unfree;
        config
    }

    // Tests

    #[test]
    fn test_render_flake_with_packages() {
        let mut config = create_config_with_packages(
            vec!["python312".to_string(), "gcc".to_string()],
            vec!["rust-analyzer".to_string()],
        );
        config.dev_shell.shell_hook = vec!["echo Hello".to_string()];

        let flake = render_flake(&config, None);

        assert!(flake.contains("stablePackages.python312"));
        assert!(flake.contains("stablePackages.gcc"));
        assert!(flake.contains("unstablePackages.rust-analyzer"));
        assert!(flake.contains("echo Hello"));
        assert!(flake.contains("config.allowUnfree = false"));
    }

    #[test]
    fn test_render_flake_allow_unfree() {
        let config = create_config_with_allow_unfree(true);

        let flake = render_flake(&config, None);
        assert!(flake.contains("config.allowUnfree = true"));
    }

    #[test]
    fn test_render_flake_with_override_stable_package() {
        let config = create_config_with_packages(vec!["python312".to_string()], vec![]);

        let custom_url = "github:NixOS/nixpkgs/nixos-25.06";
        let flake = render_flake(&config, Some(custom_url));

        // Verify custom URL is used
        assert!(flake.contains(&format!("nixpkgs.url = \"{}\";", custom_url)));
        assert!(!flake.contains("nixpkgs.url = \"github:NixOS/nixpkgs/nixos-25.11\";"));
    }

    #[test]
    fn test_render_flake_without_override_uses_default() {
        let config = create_config_with_packages(vec!["python312".to_string()], vec![]);

        let flake = render_flake(&config, None);

        // Verify default URL is used
        assert!(flake.contains("nixpkgs.url = \"github:NixOS/nixpkgs/nixos-25.11\";"));
    }

    #[test]
    fn test_render_flake_without_dotenv() {
        let env = Env {
            dotenv: vec![],
            envvar: vec![],
        };
        let config = create_config_with_env(env);

        let flake = render_flake(&config, None);

        // Verify no dotenv loading is included
        assert!(!flake.contains("Load dotenv file"));
        assert!(flake.contains("Welcome to LazyNix DevShell!"));
    }

    #[test]
    fn test_render_flake_with_single_dotenv() {
        let env = Env {
            dotenv: vec![".env".to_string()],
            envvar: vec![],
        };
        let config = create_config_with_env(env);

        let flake = render_flake(&config, None);

        // Verify dotenv loading is included
        assert!(flake.contains("Load dotenv file: .env"));
        assert!(flake.contains("set -a"));
        assert!(flake.contains("source \"$PWD/.env\""));
        assert!(flake.contains("set +a"));
    }

    #[test]
    fn test_render_flake_with_multiple_dotenv() {
        let env = Env {
            dotenv: vec![".env".to_string(), ".env.local".to_string()],
            envvar: vec![],
        };
        let config = create_config_with_env(env);

        let flake = render_flake(&config, None);

        // Verify both dotenv files are loaded
        assert!(flake.contains("Load dotenv file: .env"));
        assert!(flake.contains("Load dotenv file: .env.local"));

        // Verify order (.env should appear before .env.local)
        let env_pos = flake.find("Load dotenv file: .env\n").unwrap();
        let env_local_pos = flake.find("Load dotenv file: .env.local").unwrap();
        assert!(env_pos < env_local_pos);
    }

    #[test]
    fn test_render_flake_with_envvar_only() {
        let env = Env {
            dotenv: vec![],
            envvar: vec![
                EnvVar {
                    name: "MY_VAR".to_string(),
                    value: "hello".to_string(),
                },
                EnvVar {
                    name: "PYTHONPATH".to_string(),
                    value: "/path/to/project".to_string(),
                },
            ],
        };
        let config = create_config_with_env(env);

        let flake = render_flake(&config, None);

        // Verify envvar export is included
        assert!(flake.contains("export MY_VAR=hello"));
        assert!(flake.contains("export PYTHONPATH=/path/to/project"));
    }

    #[test]
    fn test_render_flake_with_relative_path() {
        let env = Env {
            dotenv: vec!["./config/.env".to_string(), "../.env.shared".to_string()],
            envvar: vec![],
        };
        let config = create_config_with_env(env);

        let flake = render_flake(&config, None);

        // Verify relative paths are resolved to $PWD
        assert!(flake.contains("$PWD/config/.env"));
        assert!(flake.contains("$PWD/../.env.shared"));
    }

    #[test]
    fn test_render_flake_with_absolute_path() {
        let env = Env {
            dotenv: vec!["/home/user/.env.global".to_string()],
            envvar: vec![],
        };
        let config = create_config_with_env(env);

        let flake = render_flake(&config, None);

        // Verify absolute path is used as-is
        assert!(flake.contains("/home/user/.env.global"));
        // Verify it's not prefixed with $PWD
        assert!(!flake.contains("$PWD/home/user/.env.global"));
    }

    #[test]
    fn test_render_flake_with_env_and_envvar() {
        let env = Env {
            dotenv: vec!["./config/.env".to_string(), "/tmp/.env.global".to_string()],
            envvar: vec![EnvVar {
                name: "MY_VAR".to_string(),
                value: "hello".to_string(),
            }],
        };
        let mut config = create_config_with_env(env);
        config.dev_shell.shell_hook = vec!["echo Hello".to_string()];

        let flake = render_flake(&config, None);

        // Verify path resolution
        assert!(flake.contains("$PWD/config/.env"));
        assert!(flake.contains("/tmp/.env.global"));

        // Verify envvar export
        assert!(flake.contains("export MY_VAR=hello"));

        // Verify order: dotenv → envvar → user shellHook
        let dotenv_pos = flake.find("$PWD/config/.env").unwrap();
        let envvar_pos = flake.find("export MY_VAR").unwrap();
        let user_hook_pos = flake.find("echo Hello").unwrap();
        assert!(dotenv_pos < envvar_pos);
        assert!(envvar_pos < user_hook_pos);
    }

    #[test]
    fn test_path_resolution_functions() {
        // Test is_absolute_path
        assert!(is_absolute_path("/home/user/.env"));
        assert!(is_absolute_path("/tmp/.env"));
        assert!(!is_absolute_path(".env"));
        assert!(!is_absolute_path("./config/.env"));
        assert!(!is_absolute_path("../.env"));

        // Test resolve_path for absolute paths
        assert_eq!(resolve_path("/home/user/.env"), "/home/user/.env");

        // Test resolve_path for relative paths
        assert_eq!(resolve_path(".env"), "$PWD/.env");
        assert_eq!(resolve_path("./config/.env"), "$PWD/config/.env");
        assert_eq!(resolve_path("../.env"), "$PWD/../.env");
    }

    #[test]
    fn test_render_test_execution_empty() {
        let tests: Vec<String> = vec![];
        let script = render_test_execution(&tests);
        assert_eq!(script, "");
    }

    #[test]
    fn test_render_test_execution_single_command() {
        let tests = vec!["pytest".to_string()];
        let script = render_test_execution(&tests);

        assert!(script.contains("pytest"));
        assert!(script.contains("TESTS_FAILED"));
        assert!(script.contains("TESTS_PASSED"));
        assert!(script.contains("Running tests"));
        assert!(script.contains("[PASS]"));
        assert!(script.contains("[FAIL]"));
        assert!(script.contains("exit 1"));
    }

    #[test]
    fn test_render_test_execution_multiple_commands() {
        let tests = vec![
            "pytest".to_string(),
            "cargo test".to_string(),
            "npm run test".to_string(),
        ];
        let script = render_test_execution(&tests);

        // すべてのコマンドが含まれることを確認
        assert!(script.contains("pytest"));
        assert!(script.contains("cargo test"));
        assert!(script.contains("npm run test"));

        // all-run 方式の確認
        assert!(script.contains("for test_cmd in"));
        assert!(script.contains("bash -c"));
        assert!(script.contains("Test Results:"));
    }

    #[test]
    fn test_render_flake_with_test() {
        let mut config = create_default_config();
        config.dev_shell.test = vec!["pytest".to_string(), "cargo test".to_string()];

        let flake = render_flake(&config, None);

        // テスト実行ロジックが含まれていることを確認
        assert!(flake.contains("if [ \"$LAZYNIX_TEST_MODE\" = \"1\" ]"));
        assert!(flake.contains("Running tests"));
        assert!(flake.contains("pytest"));
        assert!(flake.contains("cargo test"));
        assert!(flake.contains("TESTS_FAILED"));
        assert!(flake.contains("TESTS_PASSED"));
    }

    #[test]
    fn test_render_flake_without_test() {
        let config = create_default_config(); // test は空

        let flake = render_flake(&config, None);

        // テスト実行ロジックが含まれないことを確認
        assert!(!flake.contains("LAZYNIX_TEST_MODE"));
        assert!(!flake.contains("TESTS_FAILED"));
        assert!(flake.contains("Welcome to LazyNix DevShell!"));
    }

    #[test]
    fn test_render_flake_with_empty_shell_alias() {
        let config = create_default_config();
        let flake = render_flake(&config, None);

        assert!(!flake.contains("Load shell aliases"));
        assert!(!flake.contains("grep '^alias '"));
    }

    #[test]
    fn test_render_shell_alias_single_file() {
        let mut config = create_default_config();
        config.dev_shell.shell_alias = vec!["./aliases.sh".to_string()];

        let flake = render_flake(&config, None);

        assert!(flake.contains("Load shell aliases: ./aliases.sh"));
        assert!(flake.contains("$PWD/aliases.sh"));
        assert!(flake.contains("grep '^alias '"));
    }

    #[test]
    fn test_render_shell_alias_multiple_files() {
        let mut config = create_default_config();
        config.dev_shell.shell_alias =
            vec!["./aliases.sh".to_string(), "~/.my_aliases".to_string()];

        let flake = render_flake(&config, None);

        assert!(flake.contains("Load shell aliases: ./aliases.sh"));
        assert!(flake.contains("$PWD/aliases.sh"));
        assert!(flake.contains("Load shell aliases: ~/.my_aliases"));
        assert!(flake.contains("$HOME/.my_aliases"));
    }

    #[test]
    fn test_render_shell_alias_absolute_path() {
        let mut config = create_default_config();
        config.dev_shell.shell_alias = vec!["/etc/aliases.sh".to_string()];

        let flake = render_flake(&config, None);

        assert!(flake.contains("/etc/aliases.sh"));
        assert!(!flake.contains("$PWD/etc/aliases.sh"));
    }

    #[test]
    fn test_render_shell_alias_tilde_expansion() {
        let mut config = create_default_config();
        config.dev_shell.shell_alias = vec!["~/.aliases".to_string()];

        let flake = render_flake(&config, None);

        assert!(flake.contains("$HOME/.aliases"));
        assert!(!flake.contains("~/.aliases\" 2>/dev/null")); // Should be expanded
    }

    #[test]
    fn test_shell_hook_integration_order() {
        let mut config = create_default_config();
        config.dev_shell.env = Some(Env {
            dotenv: vec![".env".to_string()],
            envvar: vec![EnvVar {
                name: "TEST".to_string(),
                value: "value".to_string(),
            }],
        });
        config.dev_shell.shell_alias = vec!["./aliases.sh".to_string()];
        config.dev_shell.shell_hook = vec!["echo 'user hook'".to_string()];

        let flake = render_flake(&config, None);

        // Verify order: dotenv → envvar → shell_alias → user hook
        let dotenv_pos = flake.find("Load dotenv").unwrap();
        let envvar_pos = flake.find("export TEST=").unwrap();
        let alias_pos = flake.find("Load shell aliases").unwrap();
        let user_pos = flake.find("echo 'user hook'").unwrap();

        assert!(dotenv_pos < envvar_pos);
        assert!(envvar_pos < alias_pos);
        assert!(alias_pos < user_pos);
    }

    #[test]
    fn test_path_resolution_tilde() {
        // Test tilde expansion
        assert_eq!(resolve_path("~/foo"), "$HOME/foo");
        assert_eq!(resolve_path("~/.aliases"), "$HOME/.aliases");
    }
}
