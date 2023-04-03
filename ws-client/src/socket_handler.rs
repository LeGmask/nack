use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use strum_macros::{Display, EnumString};
use tokio::sync::mpsc::UnboundedSender;
use tokio_tungstenite::tungstenite::Message;

mod exec;

// #[derive(Debug, Display, EnumString)]
// #[strum(serialize_all = "snake_case")]
// pub(crate) enum Modules {
//     ,
//     GetClientsRequest,
//     RunRequest,
// }

#[derive(Debug, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
enum Modules {
    Exec
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


#[derive(Clone)]
pub struct SocketHandler {
    tx: UnboundedSender<Message>,
}

impl SocketHandler {
    pub fn new(tx: UnboundedSender<Message>) -> SocketHandler {
        SocketHandler { tx }
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
            //@todo each module return a response that need to be propagate to the socket
            Ok(Modules::Exec) => exec::run(),
            _ => tracing::error!("Invalid action"),
        };
    }


}

