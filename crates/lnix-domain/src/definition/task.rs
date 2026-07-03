use serde::{Deserialize, Serialize};

/// A user-defined task: a description and the commands to run.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskDef {
    #[serde(default)]
    pub description: Option<String>,

    pub commands: Vec<String>,
}
