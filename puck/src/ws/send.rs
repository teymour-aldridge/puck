//! Send WebSocket messages to the client.

use std::io::Write;

use super::{frame::Frame, message::Message};

/// Send a message to the provided stream.
pub(crate) fn send(stream: impl Write, msg: Message) -> Result<(), SendFrameError> {
    send_frame(stream, Frame::from(msg))
}

pub(crate) fn send_frame(mut stream: impl Write, frame: Frame) -> Result<(), SendFrameError> {
    frame.format(&mut stream)?;
    stream.write_all(frame.decoded())?;

    Ok(())
}

#[derive(thiserror::Error, Debug)]
/// An error encountered when sending a frame.
pub enum SendFrameError {
    #[error("error encoding the frame")]
    /// Indicates that the frame in question could not be encoded.
    EncodeFrameError,
    #[error("io error")]
    /// Indicates that there was an IO error when trying to handle this frame.
    IoError,
}

impl From<std::io::Error> for SendFrameError {
    fn from(_: std::io::Error) -> Self {
        Self::IoError
    }
}
