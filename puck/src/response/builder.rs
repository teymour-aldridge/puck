use std::collections::HashMap;

use crate::{
    request::{Body, Method},
    Response,
};

#[derive(Derivative, Debug)]
#[derivative(Default(new = "true"))]
pub struct ResponseBuilder {
    headers: HashMap<String, String>,
    body: Option<Body>,
    status: Option<u16>,
    reason: Option<String>,
    method: Option<Method>,
}

impl ResponseBuilder {
    pub fn header(mut self, key: impl ToString, value: impl ToString) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    pub fn headers(mut self, new_headers: impl IntoIterator<Item = (String, String)>) -> Self {
        self.headers.extend(new_headers);
        self
    }

    pub fn body(mut self, body: impl Into<Body>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn status(mut self, code: u16, reason: impl ToString) -> Self {
        self.status = Some(code);
        self.reason = Some(reason.to_string());
        self
    }

    pub fn build(self) -> Response {
        Response {
            headers: self.headers,
            body: self.body.unwrap_or_else(Body::empty),
            status: self.status.unwrap_or(200),
            reason: self.reason.unwrap_or_default(),
        }
    }
}
