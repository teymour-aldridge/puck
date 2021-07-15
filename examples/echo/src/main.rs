//! A WebSocket echo server.

use lunatic::net::TcpStream;
use puck::{
    app::{Handler, Route},
    at,
    body::Body,
    run_app,
    ws::{message::Message, websocket::WebSocket},
    Request, Response,
};

pub fn echo(mut ws: WebSocket<TcpStream>) {
    // note that this will *never* return `None`
    while let Ok(next) = ws.next().unwrap() {
        match next {
            Message::Text(_) | Message::Binary(_) => {
                let _ = ws.send(next);
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

fn main() {
    run_app!(
        puck::app::App::new()
            .route(Route::new(
                at!(""),
                Handler::Http(|request, _| home(request)),
            ))
            .route(Route::new(
                at!("ws"),
                Handler::Ws(|websocket, _| echo(websocket)),
            )),
        "127.0.0.1:5051",
        ()
    )
}
