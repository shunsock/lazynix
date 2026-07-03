use serde::{Deserialize, Serialize};

use crate::values::EnvVarName;

/// The `env` section: dotenv files and inline environment variables.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Env {
    #[serde(default)]
    pub dotenv: Vec<String>,

    #[serde(default)]
    pub envvar: Vec<EnvVar>,
}

/// A single environment variable exported into the dev shell.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvVar {
    pub name: EnvVarName,
    pub value: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_dotenv_and_envvar() {
        // Arrange
        let yaml = r#"
dotenv:
  - .env
envvar:
  - name: MY_VAR
    value: hello
"#;

        // Act
        let env: Env = serde_yaml::from_str(yaml).unwrap();

        // Assert
        assert_eq!(env.dotenv, vec![".env"]);
        assert_eq!(env.envvar.len(), 1);
        assert_eq!(env.envvar[0].name.as_str(), "MY_VAR");
        assert_eq!(env.envvar[0].value, "hello");
    }

    #[test]
    fn rejects_invalid_envvar_name_at_parse_time() {
        // Arrange
        let yaml = r#"
envvar:
  - name: 123VAR
    value: test
"#;

        // Act
        let result = serde_yaml::from_str::<Env>(yaml);

        // Assert
        let message = result.unwrap_err().to_string();
        assert!(
            message.contains("Invalid environment variable name"),
            "got: {message}"
        );
    }
}
