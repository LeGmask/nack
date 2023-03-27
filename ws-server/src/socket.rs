use std::collections::HashMap;
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt, TryFutureExt};
use tokio::sync::{mpsc, RwLock};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};

type Users = Arc<RwLock<HashMap<String, mpsc::UnboundedSender<Message>>>>;

pub struct SocketHandler {
    pub connected_users: Users,
    logged_in_clients: Vec<String>,
    logged_in_admins: Vec<String>,
}

impl SocketHandler {
    pub fn new() -> SocketHandler {
        SocketHandler {
            connected_users: Users::default(),
            logged_in_clients: Vec::new(),
            logged_in_admins: Vec::new(),
        }
    }

    pub async fn handle_connection(self, ws: WebSocket, username: String) {
        println!("New connection: {}", username);

        // Split the socket into a sender and receive of messages.
        let (mut user_ws_tx, mut user_ws_rx) = ws.split();


        // Use an unbounded channel to handle buffering and flushing of messages
        // to the websocket...
        let (tx, rx) = mpsc::unbounded_channel::<Message>();
        let mut rx = UnboundedReceiverStream::new(rx);
//
        tokio::task::spawn(async move {
            while let Some(message) = rx.next().await {
                println!("Sending message: {:?}", message);
                user_ws_tx
                    .send(message)
                    .unwrap_or_else(|e| {
                        eprintln!("websocket send error: {}", e);
                    })
                    .await;
            }
        });

        // Save the sender in our list of connected users.
        self.connected_users.write().await.insert(username.clone(), tx);

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
            // user_message(&username, msg, &users).await;
        }
        // user_ws_rx stream will keep processing as long as the user stays
        // connected. Once they disconnect, then...
//     user_disconnected(&username, &users).await;
    }
}

