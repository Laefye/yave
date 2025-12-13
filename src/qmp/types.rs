use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct QMP {
    #[serde(default)]
    pub version: Option<Value>,
    #[serde(default)]
    pub capabilities: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct Greeting {
    #[serde(rename = "QMP")]
    pub qmp: QMP,
}

#[derive(Debug, Deserialize)]
pub struct CommandError {
    pub class: String,
    pub desc: String,
}

#[derive(Debug, Deserialize)]
pub struct CommandResponse {
    #[serde(default)]
    pub id: Option<Value>,
    #[serde(rename = "return")]
    pub result: Option<Value>,
    pub error: Option<CommandError>,
}

#[derive(Debug, Deserialize)]
pub struct Event {
    pub event: String,
    #[serde(default)]
    pub data: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Response {
    Greeting(Greeting),
    CommandResponse(CommandResponse),
    Event(Event),
}

#[derive(Debug, Serialize)]
pub struct EmptyCommand {
    pub execute: String,
}

#[derive(Debug, Serialize)]
pub struct CommandWithArgs {
    pub execute: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Value>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum InvokeCommand {
    Empty(EmptyCommand),
    WithArgs(CommandWithArgs),
}

impl InvokeCommand {
    pub fn empty(command: &str) -> Self {
        InvokeCommand::Empty(EmptyCommand {
            execute: command.to_string(),
        })
    }

    pub fn with_args(command: &str, arguments: Value) -> Self {
        InvokeCommand::WithArgs(CommandWithArgs {
            execute: command.to_string(),
            arguments: Some(arguments),
        })
    }

    pub fn set_vnc_password(password: &str) -> Self {
        let args = serde_json::json!({
            "password": password,
            "protocol": "vnc",
        });
        InvokeCommand::with_args("set_password", args)
    }
}
