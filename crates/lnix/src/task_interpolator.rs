/// Interpolates CLI arguments into command templates
///
/// Replaces `{{.CLI_ARGS}}` placeholder with the provided CLI arguments.
/// Arguments are joined with spaces.
///
/// # Arguments
///
/// * `command_list` - List of command strings that may contain `{{.CLI_ARGS}}` placeholder
/// * `cli_args` - List of CLI arguments to interpolate
///
/// # Returns
///
/// A new vector of commands with placeholders replaced by CLI arguments
///
/// # Examples
///
/// ```
/// let commands = vec!["uv run python {{.CLI_ARGS}}".to_string()];
/// let args = vec!["main.py".to_string(), "--verbose".to_string()];
/// let result = interpolate_command(&commands, &args);
/// assert_eq!(result, vec!["uv run python main.py --verbose"]);
/// ```
#[allow(dead_code)]
pub fn interpolate_command(command_list: &[String], cli_args: &[String]) -> Vec<String> {
    let args_str = cli_args.join(" ");
    command_list
        .iter()
        .map(|cmd| cmd.replace("{{.CLI_ARGS}}", &args_str))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpolate_with_placeholder() {
        let commands = vec!["uv run python {{.CLI_ARGS}}".to_string()];
        let args = vec!["main.py".to_string(), "--verbose".to_string()];
        let result = interpolate_command(&commands, &args);
        assert_eq!(result, vec!["uv run python main.py --verbose"]);
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
                "echo Starting app.py --debug",
                "python app.py --debug",
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
        assert_eq!(result, vec!["echo file with spaces.txt --option=value"]);
    }

    #[test]
    fn test_interpolate_multiple_placeholders() {
        let commands = vec!["echo {{.CLI_ARGS}} and {{.CLI_ARGS}} again".to_string()];
        let args = vec!["test".to_string()];
        let result = interpolate_command(&commands, &args);
        assert_eq!(result, vec!["echo test and test again"]);
    }

    #[test]
    fn test_interpolate_no_commands() {
        let commands: Vec<String> = vec![];
        let args = vec!["arg1".to_string()];
        let result = interpolate_command(&commands, &args);
        assert_eq!(result, Vec::<String>::new());
    }
}
