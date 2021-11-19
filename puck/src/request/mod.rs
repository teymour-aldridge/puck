//! HTTP requests.

use std::{
    collections::HashMap,
    io::{self, BufRead, BufReader, Read, Write},
    str::Utf8Error,
};

use url::{ParseError, Url};

use crate::body::Body;

pub mod builder;

/// The maximum number of headers which Puck will parse.
pub const MAX_HEADERS: usize = 20;

/// The new line delimiter.
pub const NEW_LINE: u8 = b'\n';

/// A HTTP request.
#[derive(Debug)]
pub struct Request {
    pub(crate) headers: HashMap<String, String>,
    pub(crate) method: Method,
    pub(crate) body: Body,
    pub(crate) url: Url,
}

impl Request {
    /// Returns a builder to produce a new `Request` with. This method panics if the URL is not
    /// valid.
    pub fn build(url: impl AsRef<str>) -> builder::RequestBuilder {
        builder::RequestBuilder::new(url)
    }

    /// Try to construct a builder from the provided URL, and return an error if the URL is invalid.
    pub fn try_build(url: impl AsRef<str>) -> Result<builder::RequestBuilder, ParseError> {
        builder::RequestBuilder::try_new(url)
    }

    /// Parse a `Request` from the provided stream (which must implement `Read` and be valid for
    /// the `'static` lifetime.) This function will block until the `Request` has been parsed.
    ///
    /// Note that if the request is empty, this will not return an error – instead it will return
    /// `Ok(None)`.
    pub fn parse(stream: impl Read + 'static) -> Result<Option<Self>, RequestParseError> {
        let mut headers = [httparse::EMPTY_HEADER; MAX_HEADERS];
        let mut req = httparse::Request::new(&mut headers);

        let mut reader = BufReader::with_capacity(10000, stream);
        let mut buf = Vec::new();

        loop {
            let bytes_read = match reader.read_until(NEW_LINE, &mut buf) {
                Ok(t) => t,
                Err(e) => {
                    return Err(From::from(e));
                }
            };
            if bytes_read == 0 {
                return Ok(None);
            }
            // todo – drop requests for headers which are too large
            let idx = buf.len() - 1;
            if idx >= 3 && &buf[idx - 3..=idx] == b"\r\n\r\n" {
                break;
            }
        }

        let _ = req.parse(&buf)?;
        let method = Method::new_from_str(req.method.ok_or(RequestParseError::MissingMethod)?);
        let headers = {
            let mut map = HashMap::new();
            for header in req.headers.iter() {
                map.insert(
                    header.name.to_string(),
                    std::str::from_utf8(header.value)?.to_string(),
                );
            }
            map
        };

        let url =
            if let Some((_, host)) = headers.iter().find(|(k, _)| k.eq_ignore_ascii_case("host")) {
                let url = req.path.ok_or(RequestParseError::InvalidUrl)?;
                if url.starts_with("http://") || url.starts_with("https://") {
                    Url::parse(url)
                } else if url.starts_with('/') {
                    Url::parse(&format!("http://{}{}", host, url))
                } else if req.method.unwrap().eq_ignore_ascii_case("connect") {
                    Url::parse(&format!("http://{}/", host))
                } else {
                    return Err(RequestParseError::InvalidUrl);
                }
                .map_err(|_| RequestParseError::InvalidUrl)?
            } else {
                return Err(RequestParseError::MissingHeader("Host".to_string()));
            };

        let body = Body::from_reader(
            reader,
            headers
                .iter()
                .find(|(key, _)| key.eq_ignore_ascii_case("content-length"))
                .and_then(|(_, len)| len.as_str().parse::<usize>().ok()),
        );

        Ok(Some(Self {
            headers,
            method,
            body,
            url,
        }))
    }

    /// Write this `Request` into the provided writer. Note that this will modify the `Request`
    /// in-place; specifically, it will empty the contents of this `Request`'s body.
    pub fn write(&mut self, write: &mut impl Write) -> io::Result<()> {
        self.method.write(write)?;
        write!(write, " {} ", self.url.path())?;
        write!(write, "HTTP/1.1\r\n")?;
        for (key, value) in &self.headers {
            write!(write, "{}: {}\r\n", key, value)?;
        }
        write!(write, "\r\n")?;

        std::io::copy(&mut self.body, write).map(drop)
    }

    /// Get a reference to the request's headers.
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    /// Get a reference to the request's method.
    pub fn method(&self) -> &Method {
        &self.method
    }

    /// Get a reference to the request's body.
    pub fn body(&self) -> &Body {
        &self.body
    }

    /// Replace the current `Body` with the supplied `Body`, returning the existing `Body`.
    pub fn replace_body(&mut self, body: impl Into<Body>) -> Body {
        let body = std::mem::replace(&mut self.body, body.into());
        self.copy_content_type_from_body();
        body
    }

    /// Take the `Body` from this request, replacing the `Request`'s body with an empty `Body`.
    pub fn take_body(&mut self) -> Body {
        self.replace_body(Body::empty())
    }

    fn copy_content_type_from_body(&mut self) {
        self.headers
            .insert("Content-Type".into(), self.body.mime.to_string());
    }

    /// Get a reference to the request's url.
    pub fn url(&self) -> &Url {
        &self.url
    }
}

#[derive(thiserror::Error, Debug)]
/// An error encountered when trying to parse a request.
pub enum RequestParseError {
    /// Couldn't parse the request in question.
    #[error("could not parse")]
    CouldNotParse(httparse::Error),
    /// A `Utf8Error` was encountered when parsing the request.
    #[error("utf8 error")]
    Utf8Error(Utf8Error),
    /// An `IoError` was encountered when parsing the request.
    #[error("io error")]
    IoError(io::Error),
    /// The URL supplied was not valid.
    #[error("the supplied url was invalid")]
    InvalidUrl,
    /// A header is missing.
    #[error("the `{0}` header is missing")]
    MissingHeader(String),
    /// The request method is missing.
    #[error("missing method")]
    MissingMethod,
}

impl From<std::io::Error> for RequestParseError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<httparse::Error> for RequestParseError {
    fn from(e: httparse::Error) -> Self {
        Self::CouldNotParse(e)
    }
}

impl From<Utf8Error> for RequestParseError {
    fn from(e: Utf8Error) -> Self {
        Self::Utf8Error(e)
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
/// The HTTP method (e.g. "GET" or "POST")
#[allow(missing_docs)]
pub enum Method {
    Get,
    Post,
    Head,
    OtherMethod(String),
}

impl Method {
    /// Create a new method from the provided string.
    pub fn new_from_str(str: &str) -> Self {
        match str.to_ascii_lowercase().as_str() {
            "get" => Self::Get,
            "post" => Self::Post,
            _ => Self::OtherMethod(str.to_string()),
        }
    }

    /// Write the given message to a TCP stream.
    pub fn write(&self, write: &mut impl Write) -> io::Result<()> {
        let to_write = match self {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Head => "HEAD /",
            Method::OtherMethod(name) => name,
        };
        write!(write, "{}", to_write)
    }
}
