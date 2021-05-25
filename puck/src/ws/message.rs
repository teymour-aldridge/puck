use lunatic::net::TcpStream;

use crate::ws::frame::Frame;

use super::frame::{OpCode, ParseFrameError};

#[derive(Debug, Clone)]
pub enum Message {
    Ping,
    Pong,
    Text(String),
    Binary(Vec<u8>),
}

impl Message {
    pub fn next(stream: TcpStream) -> Result<Self, DecodeMessageError> {
        log::trace!("trying to parse next message");
        let first = Frame::parse(stream.clone())?;

        if *first.fin() {
            return match first.op_code() {
                crate::ws::frame::OpCode::Binary => Ok(Self::Binary(first.take_decoded())),
                crate::ws::frame::OpCode::Text => Ok(Self::Text(
                    String::from_utf8(first.take_decoded())
                        .map_err(|_| DecodeMessageError::ClientProtocolViolationError)?,
                )),
                crate::ws::frame::OpCode::Ping => Ok(Self::Ping),
                crate::ws::frame::OpCode::Pong => Ok(Self::Pong),
                _ => Err(DecodeMessageError::ClientProtocolViolationError),
            };
        }

        let op_code = *first.op_code();

        let mut payload = first.take_decoded();

        loop {
            let msg = Frame::parse(stream.clone())?;

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
                    crate::ws::frame::OpCode::Ping => Ok(Self::Ping),
                    crate::ws::frame::OpCode::Pong => Ok(Self::Pong),
                    _ => Err(DecodeMessageError::ClientProtocolViolationError),
                };
            }
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum DecodeMessageError {
    #[error("the client violated the WebSocket protocol")]
    ClientProtocolViolationError,
}

impl From<ParseFrameError> for DecodeMessageError {
    fn from(_: ParseFrameError) -> Self {
        Self::ClientProtocolViolationError
    }
}
