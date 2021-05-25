//! Send WebSocket messages to the client.

use std::io::Write;

use lunatic::net::TcpStream;

use super::{frame::Frame, message::Message};

pub fn send(stream: TcpStream, msg: Message) -> Result<(), SendFrameError> {
    send_frame(stream, Frame::from(msg))
}

pub(crate) fn send_frame(mut stream: TcpStream, frame: Frame) -> Result<(), SendFrameError> {
    frame.format(&mut stream)?;
    println!("wrote header");
    stream.write_all(frame.decoded())?;
    println!("wrote contents");

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
    fn from(e: std::io::Error) -> Self {
        dbg!(e);
        Self::IoError
    }
}
