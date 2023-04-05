#![windows_subsystem = "windows"]

use futures::{SinkExt, StreamExt};
use tokio::sync::mpsc::unbounded_channel;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_tungstenite::connect_async;
use url::Url;

use socket_handler::SocketHandler;

mod socket_handler;


#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("Starting client");

    loop {
        connect().await;
        tracing::info!("Disconnected... Reconnecting in 5 seconds...");
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}

async fn connect() {
    //connect async to the socket
    let socket = match connect_async(
        Url::parse(&format!(
            "ws://{}:{}/socket/{}",
            env!("APP_DOMAIN"),
            env!("APP_PORT"),
            env!("APP_USERNAME")
        )).unwrap()).await {

        Ok((socket, _)) => socket,
        Err(e) => {
            tracing::error!("Failed to connect to websocket: {}", e);
            return;
        }
    };

    let (mut client_ws_tx, mut client_ws_rx) = socket.split();

    // create a channel to send messages to the socket
    let (tx, rx) = unbounded_channel();
    let mut rx = UnboundedReceiverStream::new(rx);

    // spawn a task to read from the socket
    tokio::spawn(async move {
        while let Some(msg) = rx.next().await {
            client_ws_tx.send(msg).await.unwrap();
        }
    });

    let socket_handler = SocketHandler::new(tx.clone());

    // processing messages from the socket
    while let Some(msg) = client_ws_rx.next().await {
        // verify that the message is valid
        let msg = match msg {
            Ok(msg) => msg,
            Err(e) => {
                tracing::error!("websocket error: {}", e);
                break;
            }
        };

        tracing::debug!("Got: {}", msg);

        // spawn a task to handle the message using a clone of the socket handler
        let socket_handler = socket_handler.clone();
        tokio::spawn(async move {
            socket_handler.handle_message(msg).await;
        });
    }
}