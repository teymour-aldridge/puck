//! A HTTP library for the Lunatic Virtual Machine.

use std::collections::HashMap;

pub use lunatic;
use lunatic::{
    net::{TcpListener, TcpStream},
    Process,
};
pub use puck_codegen::handler;

pub mod encoder;
pub mod request;
pub mod response;

pub use request::Request;
use request::{Body, Method, HTML};
pub use response::Response;

pub trait Handler {
    fn handle(stream: TcpStream);
}

pub fn serve<H: Handler>(address: &'static str) -> anyhow::Result<()> {
    let conn = TcpListener::bind(address)?;
    while let Ok(stream) = conn.accept() {
        Process::spawn_with(stream, H::handle).detach();
    }
    Ok(())
}

pub fn err_404(_: Request) -> Response {
    Response {
        headers: {
            let mut res = HashMap::new();
            res.insert("Content-Type".to_string(), HTML.to_string());
            res
        },
        body: Body::from_string("<h1>404: Not found</h1>".to_string()),
        status: 404,
        reason: "not found".to_string(),
        method: Method::Get,
    }
}
