use std::process::Command;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::socket_handler::BasicResponse;


/// Module for running commands on the host machine
/// Use :
/// ```json
/// {"action": "run_request", "data": {"target": "soft_client", "module": "exec", "params": {"command": "ls","args": ["-l", "-a"]}}}
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Exec {
    command: String,
    args: Vec<String>,
}

impl Exec {
    pub fn new(params: Value) -> Exec {
        let params: Exec = match serde_json::from_value(params) {
            Ok(params) => params,
            Err(e) => panic!("Error: {}", e),
        };
        params
    }

    pub fn run(&self) -> BasicResponse {
        tracing::debug!("Running command: {}", self.command);
        tracing::debug!("With args: {:?}", self.args);
        let output = self.run_command().unwrap();
        tracing::debug!("Output: {}", output);

        BasicResponse::new(
            "run_response".to_string(),
            json!({
                "request": {
                    "module": "exec",
                    "command": self.command,
                    "args": self.args,
                },
                "output": output,
            }),
        )
    }

    fn run_command(&self) -> Result<String, Box<dyn std::error::Error>> {
        let output = Command::new(&self.command)
            .args(&self.args)
            .output()?;
        Ok(String::from_utf8(output.stdout)?)
    }
}

