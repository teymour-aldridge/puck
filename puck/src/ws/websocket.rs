use std::io::{Read, Write};

use log::trace;

use super::{frame::Frame, message::Message, send::send_frame};

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct WebSocket<S: Read + Write + Clone> {
    #[derivative(Debug = "ignore")]
    stream: S,
    state: WebSocketState,
}

#[derive(Debug, Copy, Clone)]
pub enum WebSocketState {
    Open,
    Closed,
}

impl<S> WebSocket<S>
where
    S: Read + Write + Clone,
{
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            state: WebSocketState::Open,
        }
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
pub enum NextMessageError {
    #[error("malformed client")]
    ClientError,
    #[error("the connection has been closed")]
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
