use std::{
    fmt,
    io::{Read, Write},
};

use log::trace;

use super::{
    frame::Frame,
    message::Message,
    send::{self, send_frame, SendFrameError},
};

#[derive(Clone)]
/// A WebSocket connection over a duplex stream.
pub struct WebSocket<S: Read + Write + Clone> {
    stream: S,
    state: WebSocketState,
}

impl<S: Read + Write + Clone> fmt::Debug for WebSocket<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WebSocket")
            .field("state", &self.state)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Copy, Clone)]
/// The state of the WebSocket connection (either open or closed).
pub enum WebSocketState {
    /// The connection is open.
    Open,
    /// The connection has been closed.
    Closed,
}

impl<S> WebSocket<S>
where
    S: Read + Write + Clone,
{
    /// Create a new WebSocket connection listening on the provided stream.
    pub fn new(stream: S) -> Self {
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
    pub fn send_to_stream(stream: S, message: Message) -> Result<(), SendFrameError> {
        send::send(stream, message)
    }

    /// Return the underlying stream.
    pub fn clone_stream(&self) -> S {
        self.stream.clone()
    }
}

impl<S: Read + Write + Clone> Iterator for WebSocket<S> {
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

impl<S> Drop for WebSocket<S>
where
    S: Read + Write + Clone,
{
    fn drop(&mut self) {
        match self.state {
            WebSocketState::Open => {
                send_close_frame(self.stream.clone());
            }
            WebSocketState::Closed => {}
        }
    }
}

#[derive(thiserror::Error, Debug)]
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
