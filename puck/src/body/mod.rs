use std::io::{BufRead, Cursor, Read};

use self::mime::{Mime, BYTE_STREAM};

pub mod mime;

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
