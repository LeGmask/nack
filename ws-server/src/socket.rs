use std::collections::HashMap;
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt, TryFutureExt};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};

type Users = Arc<RwLock<HashMap<String, mpsc::UnboundedSender<Message>>>>;
type Usernames = Arc<RwLock<Vec<String>>>;

#[derive(Clone)]
pub struct SocketHandler {
    pub connected_users: Users,
    logged_in_clients: Usernames,
    logged_in_admins: Usernames,
}

impl SocketHandler {
    pub fn new() -> SocketHandler {
        SocketHandler {
            connected_users: Users::default(),
            logged_in_clients: Usernames::default(),
            logged_in_admins: Usernames::default()
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
        self.connected_users.write().await.insert(username.clone(), tx);

        // Return a `Future` that is basically a state machine managing
        // this specific user's connection.

        // Every time the user sends a message, broadcast it to
        // all other users...
        while let Some(result) = user_ws_rx.next().await {
            println!("Received message: {:?}", result);
            let msg = match result {
                Ok(msg) => msg,
                Err(e) => {
                    eprintln!("websocket error(uid={}): {}", username, e);
                    break;
                }
            };
            self.send_message(&username, msg).await;
        }

        // user_ws_rx stream will keep processing as long as the user stays
        // connected. Once they disconnect, then...
        self.handle_disconnection(&username).await;
    }

    async fn send_message(&self, username: &String, msg: Message) {
        let msg = if let Ok(s) = msg.to_str() {
            s
        } else {
            return;
        };

        let new_msg = format!("<User#{}>: {}", username, msg);

        // New message from this user, send it to everyone else (except same uid)...
        for (&ref uid, tx) in self.connected_users.read().await.iter() {
            println!("Sending message to: {}", uid);
            if username != uid {
                if let Err(_disconnected) = tx.send(Message::text(new_msg.clone())) {
                    // The tx is disconnected, our `user_disconnected` code
                    // should be happening in another task, nothing more to
                    // do here.
                }
            }
        }
    }

    async fn handle_disconnection(&self, username: &String) {
        self.connected_users.write().await.remove(username);

    }
}

