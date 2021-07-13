//! A `Request` builder.

use std::{collections::HashMap, convert::TryFrom};

use url::Url;

use crate::{body::Body, Request};

use super::Method;

#[derive(Debug)]
/// A struct used to build HTTP requests.
pub struct RequestBuilder {
    pub(crate) headers: HashMap<String, String>,
    pub(crate) method: Option<Method>,
    pub(crate) body: Option<Body>,
    pub(crate) url: Url,
}

impl RequestBuilder {
    /// Construct a new `Request` pointing to the provided URL. If the URL is invalid, this method
    /// will panic.
    pub fn new(url: impl AsRef<str>) -> Self {
        Self::try_new(url).expect("`RequestBuilder` failed to parse the provided URL")
    }

    /// Construct a new `Request` using the provided URL, returning an error if the URL is not
    /// valid.
    pub fn try_new(url: impl AsRef<str>) -> Result<Self, url::ParseError> {
        Ok(Self {
            headers: HashMap::new(),
            method: None,
            body: None,
            url: TryFrom::try_from(url.as_ref())?,
        })
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

    /// Build this `Request`, and panic if it is not possible to do so.
    pub fn build(self) -> Request {
        self.try_build()
            .expect("a request method was not provided to `RequestBuilder`.")
    }

    /// Try to build this request, returning an error if the operation fails.
    pub fn try_build(self) -> Result<Request, TryBuildError> {
        Ok(Request {
            headers: self.headers,
            method: self
                .method
                .map(Ok)
                .unwrap_or(Err(TryBuildError::MethodNotProvided))?,
            body: self.body.unwrap_or_else(Body::empty),
            url: self.url,
        })
    }
}

#[derive(thiserror::Error, Debug, Clone)]
/// An error encountered when attempting to construct a `Request`.
pub enum TryBuildError {
    #[error("method not provided")]
    /// The request method was not provided.
    MethodNotProvided,
}
