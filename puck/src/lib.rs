//! An HTTP library for the Lunatic Virtual Machine.

#![deny(missing_debug_implementations, unused_must_use, missing_docs)]

use std::{collections::HashMap, io::Write, net::ToSocketAddrs};

#[cfg(test)]
mod regressions;

use body::{mime::HTML, Body};
pub use puck_codegen::handler;

pub use anyhow;
pub use request::Request;
pub use response::Response;

use response::encoder::Encoder;

pub mod body;
pub mod request;
pub mod response;
pub mod ws;

/// Used to handle requests.
pub trait Handler {
    /// Bind to given address and handle results, returning a result as needed.
    fn handle<ADDRESS>(address: ADDRESS) -> anyhow::Result<()>
    where
        ADDRESS: ToSocketAddrs;
}

/// Serve the given handler using the provided address.
pub fn serve<H: Handler, ADDRESS: ToSocketAddrs>(address: ADDRESS) -> anyhow::Result<()> {
    H::handle(address)
}

/// Return an error 404 not found response.
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
    }
}

/// Return a `400` error response.
pub fn err_400() -> Response {
    Response {
        headers: {
            let mut res = HashMap::new();
            res.insert("Content-Type".to_string(), HTML.to_string());
            res
        },
        body: Body::from_string("<h1>400: bad request</h1>".to_string()),
        status: 400,
        reason: "bad request".to_string(),
    }
}

/// Write the given response to a writable TCP stream.
pub fn write_response(res: Response, stream: impl Write) {
    let mut encoder = Encoder::new(res);
    encoder.write_tcp_stream(stream).unwrap();
}
