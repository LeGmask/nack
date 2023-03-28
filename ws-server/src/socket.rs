use futures_util::{SinkExt, StreamExt, TryFutureExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};

use requests_handler::RequestsHandler;

use crate::requests_handler;

#[derive(Clone)]
pub struct SocketHandler {
    requests_handler: RequestsHandler,
}

impl SocketHandler {
    pub fn new() -> SocketHandler {
        SocketHandler {
            requests_handler: RequestsHandler::new(),
        }
    }

    pub async fn handle_connection(self, ws: WebSocket, username: String) {
        // Split the socket into a sender and receive of messages.
        let (mut user_ws_tx, mut user_ws_rx) = ws.split();


        // Use an unbounded channel to handle buffering and flushing of messages
        // to the websocket...
        let (tx, rx) = mpsc::unbounded_channel::<Message>();
        let mut rx = UnboundedReceiverStream::new(rx);

        tokio::task::spawn(async move {
            while let Some(message) = rx.next().await {
                user_ws_tx
                    .send(message)
                    .unwrap_or_else(|e| {
                        eprintln!("websocket send error: {}", e);
                    })
                    .await;
            }
        });

        // Save the sender in our list of connected users.
        self.requests_handler.handle_new_socket_connection(&username, &tx).await;

        // Return a `Future` that is basically a state machine managing
        // this specific user's connection.

        // Every time the user sends a message, broadcast it to
        // all other users...
        while let Some(result) = user_ws_rx.next().await {
            let msg = match result {
                Ok(msg) => msg,
                Err(e) => {
                    eprintln!("websocket error(uid={}): {}", username, e);
                    break;
                }
            };

            if msg.is_text() {
                self.requests_handler.handle_request(msg, &username).await;
            }
        }

        // user_ws_rx stream will keep processing as long as the user stays
        // connected. Once they disconnect, then...
        self.requests_handler.handle_disconnected_socket(&username).await;
    }
}

