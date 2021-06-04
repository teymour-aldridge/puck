//! A WebSocket echo server.

use lunatic::net::TcpStream;
use puck::{
    request::Body,
    serve,
    ws::{message::Message, send::send, websocket::WebSocket},
    Request, Response,
};

pub fn echo(_: Request, stream: TcpStream) {
    let mut ws = WebSocket::new(stream.clone());

    // note that this will *never* return `None`
    while let Ok(next) = ws.next().unwrap() {
        match next {
            Message::Text(_) | Message::Binary(_) => {
                send(stream.clone(), next).unwrap();
            }
            // the `WebSocket` struct handles returning pings and pongs, so you don't have to
            _ => {}
        }
    }

    // you also don't need to close the connection - this is done automatically
}

pub fn home(_: Request) -> Response {
    Response::build()
        .header("Content-Type", "text/html")
        .body(Body::from_string("<h1>Hello World!</h1>"))
        .build()
}

#[puck::handler(
    handle(at = "/", call = "home"),
    handle(at = "/ws", call = "echo", web_socket = true)
)]
pub struct App;

fn main() {
    serve::<App, &str>("127.0.0.1:5051").expect("error running server");
}
