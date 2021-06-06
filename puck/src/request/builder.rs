use std::{collections::HashMap, convert::TryFrom};

use url::Url;

use crate::{body::Body, Request};

use super::Method;

#[derive(Debug)]
pub struct RequestBuilder {
    pub headers: HashMap<String, String>,
    pub method: Option<Method>,
    pub body: Option<Body>,
    pub url: Url,
}

impl RequestBuilder {
    /// Construct a new `Request` pointing to the provided URL. If the URL is invalid, this method
    /// will panic.
    pub fn new(url: impl AsRef<str>) -> Self {
        Self {
            headers: HashMap::new(),
            method: None,
            body: None,
            url: TryFrom::try_from(url.as_ref()).expect("invalid url supplied to `RequestBuilder`"),
        }
    }

    /// Add a new HTTP header to this `Request`.
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Add a series of new HTTP headers from the provided iterator to this request. This function
    /// accepts anything implementing `IntoIterator<Item = (String, String)>`.
    pub fn headers(mut self, new_headers: impl IntoIterator<Item = (String, String)>) -> Self {
        self.headers.extend(new_headers);
        self
    }

    /// Attach a `Body` to this `Request`.
    pub fn body(mut self, body: impl Into<Body>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// Attach a new method to this HTTP request.
    pub fn method(mut self, method: impl Into<Method>) -> Self {
        self.method = Some(method.into());
        self
    }

    /// Try to build this `Request`, panicking if it is not possible to do so.
    pub fn build(self) -> Request {
        Request {
            headers: self.headers,
            method: self
                .method
                .expect("a request method was not provided to `RequestBuilder`."),
            body: self.body.unwrap_or_else(Body::empty),
            url: self.url,
        }
    }
}
