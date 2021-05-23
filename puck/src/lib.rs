//! A HTTP library for the Lunatic Virtual Machine.

use std::collections::HashMap;

#[macro_use]
extern crate derivative;

pub use anyhow;
pub use lunatic;

pub use puck_codegen::handler;

pub mod encoder;
pub mod request;
pub mod response;
pub mod route;

pub use request::Request;
use request::{Body, Method, HTML};
pub use response::Response;

pub trait Handler {
    fn handle(address: &'static str) -> anyhow::Result<()>;
}

pub fn serve<H: Handler>(address: &'static str) -> anyhow::Result<()> {
    H::handle(address)
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
        method: Method::Get,
    }
}
