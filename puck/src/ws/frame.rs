//! WebSocket frame parsing.

use std::io::{BufReader, Read, Write};

use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};

use super::message::Message;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub(crate) fin: bool,
    pub(crate) rsv1: bool,
    pub(crate) rsv2: bool,
    pub(crate) rsv3: bool,
    pub(crate) op_code: OpCode,
    pub(crate) decoded: Vec<u8>,
}

impl Frame {
    pub fn parse(stream: impl Read) -> Result<Self, ParseFrameError> {
        let mut bufread = BufReader::new(stream);

        let (first, second) = {
            let mut buffer = [0_u8; 2];
            if bufread.read(&mut buffer)? != 2 {
                return Err(ParseFrameError::InsufficientData);
            }
            (buffer[0], buffer[1])
        };

        let fin = first & 0x80 != 0;

        let rsv1 = first & 0x40 != 0;
        let rsv2 = first & 0x20 != 0;
        let rsv3 = first & 0x10 != 0;

        let op_code = match first & 0x0F {
            0 => OpCode::Continue,
            1 => OpCode::Text,
            2 => OpCode::Binary,
            _i @ 3..=7 => OpCode::Reserved,
            8 => OpCode::Terminate,
            9 => OpCode::Ping,
            10 => OpCode::Pong,
            _i @ 11..=15 => OpCode::Reserved,
            _ => return Err(ParseFrameError::InvalidOpCode),
        };

        let masked = second & 0x80 != 0;

        let payload_length = match second & 0x7F {
            126 => bufread.read_uint::<NetworkEndian>(2)?,
            127 => bufread.read_uint::<NetworkEndian>(8)?,
            i => i as u64,
        };

        if payload_length > 0 && !masked {
            return Err(ParseFrameError::MaskNotSet);
        }

        let decoded = if payload_length > 0 {
            let mut masking_key = [0_u8; 4];
            if bufread.read(&mut masking_key)? != 4 {
                return Err(ParseFrameError::InsufficientData);
            }

            let mut encoded = Vec::with_capacity(payload_length as usize);

            let n = bufread.take(payload_length).read_to_end(&mut encoded)?;
            if n != payload_length as usize {
                if !fin && payload_length > n as u64 {
                    return Err(ParseFrameError::WaitForNextFrame(payload_length - n as u64));
                }
                return Err(ParseFrameError::IoError);
            }

            let mut decoded = vec![0_u8; payload_length as usize];
            for i in 0..encoded.len() {
                decoded[i] = encoded[i] ^ masking_key[i % 4];
            }

            decoded
        } else {
            vec![]
        };

        Ok(Self {
            fin,
            rsv1,
            rsv2,
            rsv3,
            op_code,
            decoded,
        })
    }

    /// Get a reference to the frame's fin.
    pub fn fin(&self) -> &bool {
        &self.fin
    }

    /// Get a reference to the frame's op code.
    pub fn op_code(&self) -> &OpCode {
        &self.op_code
    }

    /// Get a reference to the frame's decoded.
    pub fn decoded(&self) -> &Vec<u8> {
        &self.decoded
    }

    pub fn take_decoded(self) -> Vec<u8> {
        self.decoded
    }

    pub(crate) fn format(&self, to: &mut impl Write) -> std::io::Result<()> {
        let code = self.op_code.code();

        let one = code
            | if self.fin { 0x80 } else { 0 }
            | if self.rsv1 { 0x40 } else { 0 }
            | if self.rsv2 { 0x20 } else { 0 }
            | if self.rsv3 { 0x10 } else { 0 };

        let two = { Self::format_length(self.decoded.len() as u64) };

        to.write_all(&[one, two])?;

        let len = self.decoded.len();

        if len >= 126_usize && len < u16::MAX as usize {
            to.write_u16::<NetworkEndian>(len as u16)?;
        } else if len >= 126 {
            to.write_u64::<NetworkEndian>(len as u64)?;
        }

        Ok(())
    }

    pub(crate) fn format_length(len: u64) -> u8 {
        if len < 126 {
            len as u8
        } else if len < u16::MAX as u64 {
            126
        } else {
            127
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum OpCode {
    Continue,
    Binary,
    Text,
    Reserved,
    Terminate,
    Ping,
    Pong,
}

impl OpCode {
    fn code(self) -> u8 {
        match self {
            OpCode::Continue => 0,
            OpCode::Binary => 2,
            OpCode::Text => 1,
            OpCode::Reserved => 3,
            OpCode::Terminate => 8,
            OpCode::Ping => 9,
            OpCode::Pong => 10,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ParseFrameError {
    #[error("mask not set")]
    MaskNotSet,
    #[error("io error")]
    IoError,
    #[error("invalid op code")]
    InvalidOpCode,
    #[error("not enough data supplied")]
    InsufficientData,
    #[error("wait for next frame")]
    WaitForNextFrame(u64),
}

impl From<std::io::Error> for ParseFrameError {
    fn from(_: std::io::Error) -> Self {
        Self::IoError
    }
}

impl From<Message> for Frame {
    fn from(msg: Message) -> Self {
        Self {
            fin: true,
            rsv1: false,
            rsv2: false,
            rsv3: false,
            op_code: match msg {
                Message::Ping(_) => OpCode::Ping,
                Message::Pong(_) => OpCode::Pong,
                Message::Text(_) => OpCode::Text,
                Message::Binary(_) => OpCode::Binary,
            },
            decoded: match msg {
                Message::Text(string) => string.into_bytes(),
                Message::Binary(bin) => bin,
                Message::Ping(payload) | Message::Pong(payload) => payload.unwrap_or_default(),
            },
        }
    }
}

#[cfg(test)]
mod test_parse_frames {
    use std::io::Cursor;

    use lunatic::Process;

    use crate::ws::frame::Frame;

    fn test_parse_frames(_: ()) {
        assert_eq!(
            Frame::parse(Cursor::new([137, 0,])).unwrap(),
            Frame {
                fin: true,
                rsv1: false,
                rsv2: false,
                rsv3: false,
                op_code: crate::ws::frame::OpCode::Ping,
                decoded: vec![]
            }
        );

        assert_eq!(
            Frame::parse(Cursor::new([138, 0,])).unwrap(),
            Frame {
                fin: true,
                rsv1: false,
                rsv2: false,
                rsv3: false,
                op_code: crate::ws::frame::OpCode::Pong,
                decoded: vec![]
            }
        );
    }

    #[test]
    fn test() {
        Process::spawn_with((), test_parse_frames).join().unwrap();
    }
}
