use std::collections::HashMap;

use crate::request::{Body, Method};

use self::builder::ResponseBuilder;

pub mod builder;

/// A HTTP response.
#[derive(Debug)]
pub struct Response {
    pub(crate) headers: HashMap<String, String>,
    pub(crate) body: Body,
    pub(crate) status: u16,
    pub(crate) reason: String,
    #[allow(unused)]
    pub(crate) method: Method,
}

impl Response {
    fn copy_content_type_from_body(&mut self) {
        self.headers
            .insert("Content-Type".into(), self.body.mime.to_string());
    }
    pub fn replace_body(&mut self, body: impl Into<Body>) -> Body {
        let body = std::mem::replace(&mut self.body, body.into());
        self.copy_content_type_from_body();
        body
    }
    pub fn take_body(&mut self) -> Body {
        self.replace_body(Body::empty())
    }

    pub fn build() -> ResponseBuilder {
        ResponseBuilder::new()
    }
}
