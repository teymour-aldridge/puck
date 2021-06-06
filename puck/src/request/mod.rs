use std::{
    borrow::Cow,
    collections::HashMap,
    fmt::Display,
    io::{self, BufRead, BufReader, Cursor, Read, Write},
    str::Utf8Error,
};

use url::Url;

pub mod builder;

pub const MAX_HEADERS: usize = 20;
pub const NEW_LINE: u8 = b'\n';

/// A HTTP request.
#[derive(Debug)]
pub struct Request {
    pub headers: HashMap<String, String>,
    pub method: Method,
    pub body: Body,
    pub url: Url,
}

impl Request {
    /// Returns a builder to produce a new `Request` with.
    pub fn build(url: impl AsRef<str>) -> builder::RequestBuilder {
        builder::RequestBuilder::new(url)
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
                .find(|(key, _)| key.eq_ignore_ascii_case("content-type"))
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
}

#[derive(thiserror::Error, Debug)]
/// An error encountered when trying to parse a request.
pub enum RequestParseError {
    #[error("could not parse")]
    CouldNotParse(httparse::Error),
    #[error("utf8 error")]
    Utf8Error(Utf8Error),
    #[error("io error")]
    IoError(io::Error),
    #[error("the supplied url was invalid")]
    InvalidUrl,
    #[error("the `{0}` header is missing")]
    MissingHeader(String),
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
pub enum Method {
    Get,
    Post,
    Head,
    OtherMethod(String),
}

impl Method {
    pub fn new_from_str(str: &str) -> Self {
        match str.to_ascii_lowercase().as_str() {
            "get" => Self::Get,
            "post" => Self::Post,
            _ => Self::OtherMethod(str.to_string()),
        }
    }

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

#[derive(Derivative)]
#[derivative(Debug)]
#[cfg_attr(feature = "fuzzing", derive(DefaultMutator, ToJson, FromJson))]
/// An HTTP `Body`. This struct contains an IO source, from which the `Body`'s contents can be read,
/// as well as the MIME type of the contents of the `Body`.
pub struct Body {
    #[derivative(Debug = "ignore")]
    reader: Box<dyn BufRead + 'static>,
    pub(crate) mime: Mime,
    pub(crate) length: Option<usize>,
    bytes_read: usize,
}

impl Body {
    pub fn empty() -> Self {
        Self {
            reader: Box::new(Cursor::new(b"")),
            mime: BYTE_STREAM,
            length: Some(0),
            bytes_read: 0,
        }
    }

    /// Construct a new `Body` from the provided reader (which should implement `BufRead`). Note
    /// that if you can, you should ideally supply `content_length`.
    pub fn from_reader(reader: impl BufRead + 'static, content_length: Option<usize>) -> Self {
        Self {
            reader: Box::new(reader),
            mime: BYTE_STREAM,
            length: content_length,
            bytes_read: 0,
        }
    }

    /// Create a new `Body` from the provided string (this method accepts anything implementing
    /// `Display`.)
    ///
    /// This method is equivalent to `From<String> for Body` or `From<&str> for Body`.
    pub fn from_string(string: impl ToString) -> Self {
        let string = string.to_string();
        let length = Some(string.len());
        Self {
            reader: Box::new(Cursor::new(string)),
            mime: BYTE_STREAM,
            length,
            bytes_read: 0,
        }
    }

    /// Reads from the underlying IO source, and returns the result as bytes (`Vec<u8>`).
    pub fn into_bytes(mut self) -> std::io::Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(1024);
        self.read_to_end(&mut buf)?;
        Ok(buf)
    }

    /// Reads from the underlying IO source, and returns the result as a `String`.
    pub fn into_string(mut self) -> std::io::Result<String> {
        let mut result = String::with_capacity(self.length.unwrap_or(0));
        self.read_to_string(&mut result)?;
        Ok(result)
    }
}

impl From<String> for Body {
    fn from(string: String) -> Self {
        Body::from_string(string)
    }
}

impl From<&str> for Body {
    fn from(string: &str) -> Self {
        Body::from_string(string)
    }
}

impl Read for Body {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let buf = match self.length {
            None => buf,
            Some(length) if length == self.bytes_read => return Ok(0),
            Some(length) => {
                let max_length = (length - self.bytes_read).min(buf.len());
                &mut buf[0..max_length]
            }
        };
        let bytes = self.reader.read(buf)?;
        self.bytes_read += bytes;
        Ok(bytes)
    }
}

/* This code comes from https://github.com/http-rs/http-types/blob/main/src/mime/parse.rs */

#[derive(Debug, Clone)]
/// The MIME type of a request.
///
/// See [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types) for an
/// introduction to how MIME-types work.
pub struct Mime {
    pub(crate) essence: Cow<'static, str>,
    pub(crate) basetype: Cow<'static, str>,
    pub(crate) subtype: Cow<'static, str>,
    pub(crate) is_utf8: bool,
    pub(crate) params: Vec<(ParamName, ParamValue)>,
}

impl Display for Mime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format(self, f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParamName(Cow<'static, str>);

impl Display for ParamName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParamValue(Cow<'static, str>);

impl Display for ParamValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

pub const HTML: Mime = Mime {
    essence: Cow::Borrowed("text/html"),
    basetype: Cow::Borrowed("text"),
    subtype: Cow::Borrowed("html"),
    is_utf8: true,
    params: vec![],
};

pub const PLAIN: Mime = Mime {
    essence: Cow::Borrowed("text/plain"),
    basetype: Cow::Borrowed("text"),
    subtype: Cow::Borrowed("plain"),
    is_utf8: true,
    params: vec![],
};

pub const BYTE_STREAM: Mime = Mime {
    essence: Cow::Borrowed("application/octet-stream"),
    basetype: Cow::Borrowed("application"),
    subtype: Cow::Borrowed("octet-stream"),
    is_utf8: false,
    params: vec![],
};

/// Implementation of the
//// [WHATWG MIME serialization algorithm](https://mimesniff.spec.whatwg.org/#serializing-a-mime-type)
pub(crate) fn format(mime_type: &Mime, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", &mime_type.essence)?;
    if mime_type.is_utf8 {
        write!(f, ";charset=utf-8")?;
    }
    for (name, value) in mime_type.params.iter() {
        if value.0.chars().all(is_http_token_code_point) && !value.0.is_empty() {
            write!(f, ";{}={}", name, value)?;
        } else {
            let value = value
                .0
                .chars()
                .flat_map(|c| match c {
                    '"' | '\\' => EscapeMimeValue::backslash(c),
                    c => EscapeMimeValue::char(c),
                })
                .collect::<String>();
            write!(f, ";{}=\"{}\"", name, value)?;
        }
    }
    Ok(())
}

struct EscapeMimeValue {
    state: EscapeMimeValueState,
}

impl EscapeMimeValue {
    fn backslash(c: char) -> Self {
        EscapeMimeValue {
            state: EscapeMimeValueState::Backslash(c),
        }
    }

    fn char(c: char) -> Self {
        EscapeMimeValue {
            state: EscapeMimeValueState::Char(c),
        }
    }
}

#[derive(Clone, Debug)]
enum EscapeMimeValueState {
    Done,
    Char(char),
    Backslash(char),
}

impl Iterator for EscapeMimeValue {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        match self.state {
            EscapeMimeValueState::Done => None,
            EscapeMimeValueState::Char(c) => {
                self.state = EscapeMimeValueState::Done;
                Some(c)
            }
            EscapeMimeValueState::Backslash(c) => {
                self.state = EscapeMimeValueState::Char(c);
                Some('\\')
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.state {
            EscapeMimeValueState::Done => (0, Some(0)),
            EscapeMimeValueState::Char(_) => (1, Some(1)),
            EscapeMimeValueState::Backslash(_) => (2, Some(2)),
        }
    }
}

/// Validates [HTTP token code points](https://mimesniff.spec.whatwg.org/#http-token-code-point)
fn is_http_token_code_point(c: char) -> bool {
    matches!(c,
        '!'
        | '#'
        | '$'
        | '%'
        | '&'
        | '\''
        | '*'
        | '+'
        | '-'
        | '.'
        | '^'
        | '_'
        | '`'
        | '|'
        | '~'
        | 'a'..='z'
        | 'A'..='Z'
        | '0'..='9')
}

/* End "borrowed" code section. */
