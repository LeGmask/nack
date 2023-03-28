use warp::Filter;
use socket::SocketHandler;
mod socket;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let socket_handler = SocketHandler::new();

    // Turn our "state" into a new Filter...
    let socket_handler = warp::any().map(move || socket_handler.clone());

    // GET /chat -> websocket upgrade
    let chat = warp::path!("chat" / String)
        // The `ws()` filter will prepare Websocket handshake...
        .and(warp::ws())
        .and(socket_handler)
        .map(|username: String, ws: warp::ws::Ws, socket_handler: SocketHandler| {
            // This will call our function if the handshake succeeds.
            ws.on_upgrade(move |socket| socket_handler.handle_connection(socket, username))
        });

    // GET / -> index html
    // let index = warp::path::end().map(|| warp::reply::html(INDEX_HTML));

    // let routes = index.or(chat);

    warp::serve(chat).run(([127, 0, 0, 1], 3030)).await;
}