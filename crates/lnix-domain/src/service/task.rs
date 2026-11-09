//! Task-command assembly: CLI-argument interpolation.

/// Interpolates CLI arguments into command templates
///
/// Replaces `{{.CLI_ARGS}}` placeholder with the provided CLI arguments.
/// Each argument is wrapped in POSIX single quotes (see [`quote_posix`])
/// before the arguments are joined with spaces, so an argument reaches
/// the shell as a literal value and is never re-interpreted by it —
/// shell metacharacters in CLI arguments (`;`, `$()`, backticks, etc.)
/// cannot escape into command injection this way. The command template
/// itself is trusted, author-written shell and is left untouched.
///
/// # Arguments
///
/// * `command_list` - List of command strings that may contain `{{.CLI_ARGS}}` placeholder
/// * `cli_args` - List of CLI arguments to interpolate
///
/// # Returns
///
/// A new vector of commands with placeholders replaced by quoted CLI arguments
///
/// # Examples
///
/// ```
/// use lnix_domain::interpolate_command;
///
/// let commands = vec!["uv run python {{.CLI_ARGS}}".to_string()];
/// let args = vec!["main.py".to_string(), "--verbose".to_string()];
/// let result = interpolate_command(&commands, &args);
/// assert_eq!(result, vec!["uv run python 'main.py' '--verbose'"]);
/// ```
pub fn interpolate_command(command_list: &[String], cli_args: &[String]) -> Vec<String> {
    let args_str = cli_args
        .iter()
        .map(|arg| quote_posix(arg))
        .collect::<Vec<_>>()
        .join(" ");
    command_list
        .iter()
        .map(|cmd| cmd.replace("{{.CLI_ARGS}}", &args_str))
        .collect()
}

/// Wraps `arg` in POSIX single quotes so a shell treats it as one literal
/// value, regardless of any metacharacters it contains.
///
/// Embedded single quotes are escaped with the standard `'\''` sequence
/// (close quote, escaped quote, reopen quote), since a single-quoted
/// string cannot contain a literal `'`.
fn quote_posix(arg: &str) -> String {
    format!("'{}'", arg.replace('\'', r"'\''"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpolate_with_placeholder() {
        let commands = vec!["uv run python {{.CLI_ARGS}}".to_string()];
        let args = vec!["main.py".to_string(), "--verbose".to_string()];
        let result = interpolate_command(&commands, &args);
        assert_eq!(result, vec!["uv run python 'main.py' '--verbose'"]);
    }

    #[test]
    fn test_interpolate_without_placeholder() {
        let commands = vec!["cargo build".to_string()];
        let args = vec!["--release".to_string()];
        let result = interpolate_command(&commands, &args);
        assert_eq!(result, vec!["cargo build"]);
    }

    #[test]
    fn test_interpolate_empty_args() {
        let commands = vec!["uv run python {{.CLI_ARGS}}".to_string()];
        let args: Vec<String> = vec![];
        let result = interpolate_command(&commands, &args);
        assert_eq!(result, vec!["uv run python "]);
    }

    #[test]
    fn test_quote_posix_escapes_embedded_single_quote() {
        let commands = vec!["echo {{.CLI_ARGS}}".to_string()];
        let args = vec!["it's".to_string()];
        let result = interpolate_command(&commands, &args);
        assert_eq!(result, vec!["echo 'it'\\''s'".to_string()]);
    }

    #[test]
    fn test_interpolate_multiple_commands() {
        let commands = vec![
            "echo Starting {{.CLI_ARGS}}".to_string(),
            "python {{.CLI_ARGS}}".to_string(),
            "echo Finished".to_string(),
        ];
        let args = vec!["app.py".to_string(), "--debug".to_string()];
        let result = interpolate_command(&commands, &args);
        assert_eq!(
            result,
            vec![
                "echo Starting 'app.py' '--debug'",
                "python 'app.py' '--debug'",
                "echo Finished"
            ]
        );
    }

    #[test]
    fn test_interpolate_special_chars() {
        let commands = vec!["echo {{.CLI_ARGS}}".to_string()];
        let args = vec![
            "file with spaces.txt".to_string(),
            "--option=value".to_string(),
        ];
        let result = interpolate_command(&commands, &args);
        assert_eq!(result, vec!["echo 'file with spaces.txt' '--option=value'"]);
    }

    #[test]
    fn test_interpolate_multiple_placeholders() {
        let commands = vec!["echo {{.CLI_ARGS}} and {{.CLI_ARGS}} again".to_string()];
        let args = vec!["test".to_string()];
        let result = interpolate_command(&commands, &args);
        assert_eq!(result, vec!["echo 'test' and 'test' again"]);
    }

    #[test]
    fn test_interpolate_no_commands() {
        let commands: Vec<String> = vec![];
        let args = vec!["arg1".to_string()];
        let result = interpolate_command(&commands, &args);
        assert_eq!(result, Vec::<String>::new());
    }

    #[test]
    fn test_interpolate_blocks_shell_metacharacters_via_quoting() {
        let commands = vec!["echo {{.CLI_ARGS}}".to_string()];
        let args = vec![
            "; rm -rf /".to_string(),
            "$(whoami)".to_string(),
            "`whoami`".to_string(),
        ];
        let result = interpolate_command(&commands, &args);
        assert_eq!(
            result,
            vec!["echo '; rm -rf /' '$(whoami)' '`whoami`'".to_string()]
        );
    }

    /// Medium test: spawns the real `sh` process so the interpolated
    /// command is actually re-parsed by a POSIX shell. A `small` test
    /// (pure string comparison) cannot observe whether a shell would
    /// re-interpret the metacharacters, which is the exact failure mode
    /// of the original vulnerability.
    #[test]
    fn test_interpolated_command_reaches_sh_as_literal_argv_not_reinterpreted() {
        let commands = vec!["printf '%s\\n' {{.CLI_ARGS}}".to_string()];
        let args = vec!["; echo pwned".to_string(), "$(echo x)".to_string()];
        let command = interpolate_command(&commands, &args).remove(0);

        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(&command)
            .output()
            .expect("sh should be available to run this test");

        // NOTE: a bare "pwned"/"x" line means the shell re-ran args.
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "; echo pwned\n$(echo x)\n");
        assert!(!stdout.lines().any(|line| line == "pwned"));
        assert!(!stdout.lines().any(|line| line == "x"));
    }
}
