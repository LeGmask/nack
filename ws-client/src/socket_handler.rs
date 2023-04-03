mod exec;
mod open_url;
mod play_url;

use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use strum_macros::{Display, EnumString};
use tokio::sync::mpsc::UnboundedSender;
use tokio_tungstenite::tungstenite::Message;

use exec::Exec;
use open_url::OpenUrl;
use play_url::PlayUrl;

#[derive(Debug, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
enum Modules {
    Exec,
    OpenUrl,
    PlayUrl,
}

#[derive(Debug, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
enum Actions {
    Run
}

#[derive(Debug, Serialize, Deserialize)]
struct BasicRequest {
    action: String,
    data: BasicRequestData,
}

#[derive(Debug, Serialize, Deserialize)]
struct BasicRequestData {
    module: String,
    params: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BasicResponse {
    action: String,
    data: Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthRequestBody {
    app_key: String,
}

impl BasicResponse {
    fn new(action: String, data: Value) -> BasicResponse {
        BasicResponse {
            action,
            data,
        }
    }

    fn to_json_string(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}


#[derive(Clone)]
pub struct SocketHandler {
    tx: UnboundedSender<Message>,
}


impl SocketHandler {
    pub fn new(tx: UnboundedSender<Message>) -> SocketHandler {
        let socket_handler = SocketHandler { tx };
        socket_handler.auth_request();
        tracing::info!("SocketHandler created, auth request sent");
        socket_handler
    }

    fn auth_request(&self) {
        let app_key = env!("APP_KEY");

        self.tx.send(Message::text(BasicResponse::new(
            "auth_request".to_string(),
            json!({"app_key": app_key})).to_json_string()
        )).unwrap();
    }

    pub async fn handle_message(&self, message: Message) {
        if !message.is_text() {
            return;
        }

        let message: BasicRequest = match serde_json::from_str(message.to_text().unwrap()) {
            Ok(parsed_message) => parsed_message,
            Err(_) => {
                tracing::error!("Invalid JSON");
                return;
            } // If the message is not a valid JSON, ignore it
        };

        match Actions::from_str(&*message.action) {
            Ok(Actions::Run) => self.handle_run_action(message).await,
            Err(_) => tracing::error!("Invalid action"),
        }
    }

    async fn handle_run_action(&self, message: BasicRequest) {
        match Modules::from_str(&*message.data.module) {
            Ok(Modules::Exec) => self.send_response(Exec::new(message.data.params).run()),
            Ok(Modules::OpenUrl) => self.send_response(OpenUrl::new(message.data.params).run()),
            Ok(Modules::PlayUrl) => self.send_response(PlayUrl::new(message.data.params).run().await),
            _ => tracing::error!("Invalid action"),
        };
    }

    fn send_response(&self, response: BasicResponse) {
        self.tx.send(Message::text(response.to_json_string())).unwrap();
    }
}

