use std::{collections::HashMap, convert::TryFrom};

use url::Url;

use crate::Request;

use super::{Body, Method};

#[derive(Debug)]
pub struct RequestBuilder {
    pub headers: HashMap<String, String>,
    pub method: Option<Method>,
    pub body: Option<Body>,
    pub url: Url,
}

impl RequestBuilder {
    pub fn new(url: impl AsRef<str>) -> Self {
        Self {
            headers: HashMap::new(),
            method: None,
            body: None,
            url: TryFrom::try_from(url.as_ref()).expect("invalid url supplied to `RequestBuilder`"),
        }
    }

    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
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

    pub fn method(mut self, method: impl Into<Method>) -> Self {
        self.method = Some(method.into());
        self
    }

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
