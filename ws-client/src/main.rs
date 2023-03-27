use url::Url;
use tungstenite::{connect, Message};

fn main() {
    let (mut socket, _response) = connect(
        Url::parse("ws://127.0.0.1:3030/chat/soft_client").unwrap()
    ).expect("Can't connect");

    // socket.write_message(Message::Text(r#"{
    //     "action": "authenticate",
    //     "data": {
    //         "key_id": "API-KEY",
    //         "secret_key": "SECRET-KEY"
    //     }
    // }"#.into()));


    loop {
        let msg = socket.read_message().expect("Error reading message");
        println!("Received: {}", msg);
    }
}
