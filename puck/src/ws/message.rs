use std::io::Read;

use log::trace;
use serde::{Deserialize, Serialize};

use crate::ws::frame::Frame;

use super::frame::{OpCode, ParseFrameError};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// A WebSocket message.
pub enum Message {
    /// A ping.
    Ping(Option<Vec<u8>>),
    /// A pong.
    Pong(Option<Vec<u8>>),
    /// A text message.
    Text(String),
    /// A binary message.
    Binary(Vec<u8>),
}

impl Message {
    /// Parse the next message from the stream.
    pub fn next(stream: impl Read + Clone) -> Result<Self, DecodeMessageError> {
        log::trace!("trying to parse next message");
        let first = Frame::parse(stream.clone())?;

        if first.op_code() == &OpCode::Terminate {
            trace!("Client asked to close connection");
            return Err(DecodeMessageError::ClientSentCloseFrame);
        }

        if *first.fin() {
            return match first.op_code() {
                crate::ws::frame::OpCode::Binary => Ok(Self::Binary(first.take_decoded())),
                crate::ws::frame::OpCode::Text => Ok(Self::Text(
                    String::from_utf8(first.take_decoded())
                        .map_err(|_| DecodeMessageError::ClientProtocolViolationError)?,
                )),
                crate::ws::frame::OpCode::Ping => {
                    let payload = first.take_decoded();
                    Ok(Self::Ping(if !payload.is_empty() {
                        Some(payload)
                    } else {
                        None
                    }))
                }
                crate::ws::frame::OpCode::Pong => {
                    let payload = first.take_decoded();
                    Ok(Self::Pong(if !payload.is_empty() {
                        Some(payload)
                    } else {
                        None
                    }))
                }
                _ => Err(DecodeMessageError::ClientProtocolViolationError),
            };
        }

        let op_code = *first.op_code();

        let mut payload = first.take_decoded();

        loop {
            let msg = Frame::parse(stream.clone())?;

            if msg.op_code() == &OpCode::Terminate {
                trace!("Client asked to close connection");
                return Err(DecodeMessageError::ClientSentCloseFrame);
            }

            if msg.op_code() != &OpCode::Continue {
                return Err(DecodeMessageError::ClientProtocolViolationError);
            }

            let fin = *msg.fin();

            payload.extend(msg.take_decoded());

            if fin {
                return match op_code {
                    crate::ws::frame::OpCode::Binary => Ok(Self::Binary(payload)),
                    crate::ws::frame::OpCode::Text => {
                        Ok(Self::Text(String::from_utf8(payload).map_err(|_| {
                            DecodeMessageError::ClientProtocolViolationError
                        })?))
                    }
                    crate::ws::frame::OpCode::Ping => Ok(Self::Ping(if !payload.is_empty() {
                        Some(payload)
                    } else {
                        None
                    })),
                    crate::ws::frame::OpCode::Pong => Ok(Self::Pong(if !payload.is_empty() {
                        Some(payload)
                    } else {
                        None
                    })),
                    _ => Err(DecodeMessageError::ClientProtocolViolationError),
                };
            }
        }
    }
}

#[derive(thiserror::Error, Debug)]
/// An error encountered when trying to decode a WebSocket message.
#[allow(missing_docs)]
pub enum DecodeMessageError {
    #[error("the client violated the WebSocket protocol")]
    ClientProtocolViolationError,
    #[error("the client wants to close the connection")]
    ClientSentCloseFrame,
}

impl From<ParseFrameError> for DecodeMessageError {
    fn from(_: ParseFrameError) -> Self {
        Self::ClientProtocolViolationError
    }
}
