use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::socket_handler::BasicResponse;

/// Module for running commands on the host machine
/// Use :
/// ```json
/// {"action": "run_request", "data": {"target": "soft_client", "module": "open_url", "params": {"url": "https://www.youtube.com/watch?v=dQw4w9WgXcQ"}}}
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct OpenUrl {
    url: String,
}

impl OpenUrl {
    pub fn new(params: Value) -> OpenUrl {
        let params: OpenUrl = match serde_json::from_value(params) {
            Ok(params) => params,
            Err(e) => panic!("Error: {}", e),
        };
        params
    }

    pub fn run(&self) -> BasicResponse {
        tracing::debug!("Oppening url: {}", self.url);
        open::that(&self.url).unwrap();
        BasicResponse::new(
            "run_response".to_string(),
            json!({
                "request": {
                    "module": "open_url",
                    "url": self.url,
                },
                "output": "success",
            }),
        )
    }
}

