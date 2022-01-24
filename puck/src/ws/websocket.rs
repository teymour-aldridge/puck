use std::io::Write;

use log::trace;
use lunatic::net::TcpStream;

use crate::core::UsedStream;

use super::{
    frame::Frame,
    message::Message,
    send::{self, send_frame, SendFrameError},
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
/// A WebSocket connection over a duplex stream.
///
/// note: this _can_ be sent from one process to another, but it is intended that this struct
/// only be used from one process at once
#[must_use]
pub struct WebSocket {
    stream: TcpStream,
    state: WebSocketState,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy)]
/// The state of the WebSocket connection (either open or closed).
pub enum WebSocketState {
    /// The connection is open.
    Open,
    /// The connection has been closed.
    Closed,
}

impl WebSocket {
    /// Create a new WebSocket connection listening on the provided stream.
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            state: WebSocketState::Open,
        }
    }

    /// Send a message to the other party.
    pub fn send(&self, message: Message) -> Result<(), SendFrameError> {
        send::send(self.clone_stream(), message)
    }

    /// Send a message to a stream.
    pub fn send_to_stream(stream: TcpStream, message: Message) -> Result<(), SendFrameError> {
        send::send(stream, message)
    }

    /// Return the underlying stream.
    fn clone_stream(&self) -> TcpStream {
        self.stream.clone()
    }

    /// Close the WebSocket connection.
    pub fn close(self) -> Result<UsedStream, SendFrameError> {
        match self.state {
            WebSocketState::Open => {
                send_close_frame(self.stream.clone());
            }
            WebSocketState::Closed => {}
        };

        Ok(UsedStream {
            stream: Some(self.stream),
            keep_alive: false,
        })
    }

    /// You probably don't want to use this.
    pub fn make_copy(&self) -> WebSocket {
        Self {
            stream: self.stream.clone(),
            state: self.state,
        }
    }
}

impl Iterator for WebSocket {
    type Item = Result<Message, NextMessageError>;

    fn next(&mut self) -> Option<Result<Message, NextMessageError>> {
        Some(match self.state {
            WebSocketState::Open => match Message::next(self.stream.clone()) {
                Ok(msg) => {
                    if let Message::Ping(ref payload) = msg {
                        send_frame(
                            self.stream.clone(),
                            Frame {
                                fin: true,
                                rsv1: false,
                                rsv2: false,
                                rsv3: false,
                                op_code: super::frame::OpCode::Pong,
                                decoded: payload.clone().unwrap_or_default(),
                            },
                        )
                        .expect("failed to send pong");
                    }
                    Ok(msg)
                }
                Err(e) => match e {
                    super::message::DecodeMessageError::ClientProtocolViolationError => {
                        Err(NextMessageError::ClientError)
                    }
                    super::message::DecodeMessageError::ClientSentCloseFrame => {
                        self.state = WebSocketState::Closed;
                        send_close_frame(self.stream.clone());
                        Err(NextMessageError::ConnectionClosed)
                    }
                },
            },
            WebSocketState::Closed => Err(NextMessageError::ConnectionClosed),
        })
    }
}

#[derive(thiserror::Error, Debug, serde::Serialize, serde::Deserialize)]
/// An error encountered when trying to lift the next message from the stream.
pub enum NextMessageError {
    #[error("malformed client")]
    /// The client sent an invalid request.
    ClientError,
    #[error("the connection has been closed")]
    /// The connection is already closed.
    ConnectionClosed,
}

fn send_close_frame(stream: impl Write) {
    trace!("Sending close frame");
    send_frame(
        stream,
        Frame {
            fin: true,
            rsv1: false,
            rsv2: false,
            rsv3: false,
            op_code: super::frame::OpCode::Terminate,
            decoded: vec![],
        },
    )
    .expect("failed to send close frame");
}
