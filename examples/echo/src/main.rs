//! A WebSocket echo server.

use puck::{
    lunatic::net::TcpStream,
    request::Body,
    serve,
    ws::{message::Message, send},
    Request, Response,
};

pub fn echo(_: Request, stream: TcpStream) {
    loop {
        let next = Message::next(stream.clone());
        if let Ok(msg) = next {
            send::send(stream.clone(), msg).unwrap();
        } else {
            println!("{:#?}", next.unwrap_err());
        }
    }
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
    serve::<App>("127.0.0.1:5051").expect("error running server");
}
