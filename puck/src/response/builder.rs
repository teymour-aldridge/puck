use std::collections::HashMap;
use std::fmt::Debug;

use crate::{body::Body, request::Method, Response};

pub struct ResponseBuilder {
    headers: HashMap<String, String>,
    body: Option<Body>,
    status: Option<u16>,
    reason: Option<String>,
    method: Option<Method>,
}

impl Debug for ResponseBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResponseBuilder")
            .field("headers", &self.headers)
            .field("status", &self.status)
            .field("reason", &self.reason)
            .field("method", &self.method)
            .finish_non_exhaustive()
    }
}

impl Default for ResponseBuilder {
    fn default() -> Self {
        Self {
            headers: HashMap::new(),
            body: None,
            status: None,
            reason: None,
            method: None,
        }
    }
}

impl ResponseBuilder {
    /// Create a new `ResponseBuilder`. Equivalent to the `Default` implementation.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a header for this HTTP response.
    pub fn header(mut self, key: impl ToString, value: impl ToString) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    /// Set a series of headers to this HTTP response from the provided iterator.
    pub fn headers(mut self, new_headers: impl IntoIterator<Item = (String, String)>) -> Self {
        self.headers.extend(new_headers);
        self
    }

    /// Set the `Body` for this HTTP response.
    pub fn body(mut self, body: impl Into<Body>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// Set the status for this `Response`.
    pub fn status(mut self, code: u16, reason: impl ToString) -> Self {
        self.status = Some(code);
        self.reason = Some(reason.to_string());
        self
    }

    /// Build this HTTP response. This function will not panic.
    pub fn build(self) -> Response {
        Response {
            headers: self.headers,
            body: self.body.unwrap_or_else(Body::empty),
            status: self.status.unwrap_or(200),
            reason: self.reason.unwrap_or_default(),
        }
    }
}
