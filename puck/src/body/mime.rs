//! HTTP MIME types.

use std::{borrow::Cow, fmt::Display};

/* This code comes from https://github.com/http-rs/http-types/blob/main/src/mime/parse.rs */

#[derive(Debug, Clone)]
/// The MIME type of a request.
///
/// See [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types) for an
/// introduction to how MIME-types work.
#[allow(dead_code)]
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
/// [WHATWG MIME serialization algorithm](https://mimesniff.spec.whatwg.org/#serializing-a-mime-type)
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
