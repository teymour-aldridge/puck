//! Send WebSocket messages to the client.

use std::io::Write;

use super::{frame::Frame, message::Message};

pub fn send(stream: impl Write, msg: Message) -> Result<(), SendFrameError> {
    send_frame(stream, Frame::from(msg))
}

pub(crate) fn send_frame(mut stream: impl Write, frame: Frame) -> Result<(), SendFrameError> {
    frame.format(&mut stream)?;
    stream.write_all(frame.decoded())?;

    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum SendFrameError {
    #[error("error encoding the frame")]
    EncodeFrameError,
    #[error("io error")]
    IoError,
}

impl From<std::io::Error> for SendFrameError {
    fn from(_: std::io::Error) -> Self {
        Self::IoError
    }
}
