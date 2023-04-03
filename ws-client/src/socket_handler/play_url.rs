use std::io::Cursor;

use rodio::{OutputStream};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::socket_handler::BasicResponse;

/// Module for running commands on the host machine
/// Use :
/// ```json
/// {"action": "run_request", "data": {"target": "soft_client", "module": "play_url", "params": {"url": "https://drive.google.com/uc?export=download&id=1c50jZNNreeSXeaDOfoaZJ75eV8mScZFc"}}}
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct PlayUrl {
    url: String,
}

impl PlayUrl {
    pub fn new(params: Value) -> PlayUrl {
        let params: PlayUrl = match serde_json::from_value(params) {
            Ok(params) => params,
            Err(e) => panic!("Error: {}", e),
        };
        params
    }

    pub async fn run(&self) -> BasicResponse {
        tracing::debug!("Oppening url: {}", self.url);
        let file = reqwest::get(&self.url).await.unwrap();
        let cursor = Cursor::new(file.bytes().await.unwrap());


        let (_stream, handle) = OutputStream::try_default().unwrap();
        let sink = rodio::Sink::try_new(&handle).unwrap();


        sink.append(rodio::Decoder::new(cursor).unwrap());


        sink.sleep_until_end();

        BasicResponse::new(
            "run_response".to_string(),
            json!({
                "request": {
                    "module": "play_url",
                    "url": self.url,
                },
                "output": "success",
            }),
        )
    }
}

