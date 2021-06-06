use std::{
    collections::HashMap,
    io::{self, BufRead, BufReader, Read},
};

use crate::request::{MAX_HEADERS, NEW_LINE};

use self::builder::ResponseBuilder;

pub mod builder;

/// A HTTP response.
#[derive(Debug)]
#[cfg_attr(feature = "fuzzing", derive(DefaultMutator, ToJson, FromJson))]
pub struct Response {
    pub(crate) headers: HashMap<String, String>,
    pub(crate) body: Body,
    pub(crate) status: u16,
    pub(crate) reason: String,
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

    /// Obtain this `Response`'s `Body`, replacing the existing `Body` with an empty `Body`.
    pub fn take_body(&mut self) -> Body {
        self.replace_body(Body::empty())
    }

    /// Return a new `ResponseBuilder`, with which you can construct a new `Response`.
    pub fn build() -> ResponseBuilder {
        ResponseBuilder::new()
    }

    /// Attempt to parse this `Response` from a stream (anything implementing `Read` that lives for
    /// `static`.) Note that if the response is empty, this function will return Ok(None), rather
    /// than an error.
    pub fn parse(stream: impl Read + 'static) -> Result<Option<Response>, ParseResponseError> {
        let mut reader = BufReader::with_capacity(1000, stream);

        let mut headers = [httparse::EMPTY_HEADER; MAX_HEADERS];
        let mut res = httparse::Response::new(&mut headers);
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

        let _ = res.parse(&buf);

        let headers = {
            let mut map = HashMap::new();
            for header in res.headers.iter() {
                map.insert(
                    header.name.to_string(),
                    match std::str::from_utf8(header.value) {
                        Ok(t) => t,
                        Err(_) => return Err(ParseResponseError::Utf8Error),
                    }
                    .to_string(),
                );
            }
            map
        };

        let status = if let Some(status) = res.code {
            status
        } else {
            return Err(ParseResponseError::MissingStatusCode);
        };

        let reason = if let Some(reason) = res.reason {
            reason.to_string()
        } else {
            return Err(ParseResponseError::MissingReason);
        };

        let body = Body::from_reader(
            reader,
            headers
                .iter()
                .find(|(key, _)| key.eq_ignore_ascii_case("content-type"))
                .and_then(|(_, len)| len.as_str().parse::<usize>().ok()),
        );

        Ok(Some(Self {
            headers,
            body,
            status,
            reason,
        }))
    }

    /// Get a reference to the response's headers.
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    /// Get a reference to the response's status.
    pub fn status(&self) -> &u16 {
        &self.status
    }

    /// Get a reference to the response's reason.
    pub fn reason(&self) -> &str {
        self.reason.as_str()
    }
}

#[derive(thiserror::Error, Debug)]
/// An error encountered when parsing a `Response`.
pub enum ParseResponseError {
    #[error("io error")]
    IoError(io::Error),
    #[error("the status code was not supplied")]
    MissingStatusCode,
    #[error("a reason was not supplied")]
    MissingReason,
    #[error("utf8 error")]
    Utf8Error,
}

impl From<io::Error> for ParseResponseError {
    fn from(error: io::Error) -> Self {
        Self::IoError(error)
    }
}
