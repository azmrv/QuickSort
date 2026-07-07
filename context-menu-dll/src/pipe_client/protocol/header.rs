//! Fixed binary header for IPC messages.

use std::io::Cursor;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use crate::pipe_client::PipeError;

pub const MAGIC: u32 = 0x53515254; // "QSRT"
pub const PROTOCOL_VERSION: u16 = 1;
pub const MAX_MESSAGE_SIZE: u32 = 16 * 1024 * 1024; // 16 MB

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MessageHeader {
    pub magic: u32,
    pub version: u16,
    pub flags: u16,
    pub length: u32,
}

impl MessageHeader {
    pub const SIZE: usize = 4 + 2 + 2 + 4; // 12 bytes

    pub fn new(length: u32) -> Self {
        Self {
            magic: MAGIC,
            version: PROTOCOL_VERSION,
            flags: 0,
            length,
        }
    }

    pub fn encode(&self) -> Bytes {
        let mut buf = BytesMut::with_capacity(Self::SIZE);
        buf.put_u32_le(self.magic);
        buf.put_u16_le(self.version);
        buf.put_u16_le(self.flags);
        buf.put_u32_le(self.length);
        buf.freeze()
    }

    pub fn decode(data: &[u8]) -> Result<Self, PipeError> {
        if data.len() < Self::SIZE {
            return Err(PipeError::IncompleteRead {
                expected: Self::SIZE as u32,
                actual: data.len() as u32,
            });
        }

        let mut cursor = Cursor::new(data);
        let magic = cursor.get_u32_le();
        if magic != MAGIC {
            return Err(PipeError::InvalidMagic);
        }

        let version = cursor.get_u16_le();
        if version != PROTOCOL_VERSION {
            return Err(PipeError::VersionMismatch {
                expected: PROTOCOL_VERSION,
                actual: version,
            });
        }

        let flags = cursor.get_u16_le();
        let length = cursor.get_u32_le();

        if length > MAX_MESSAGE_SIZE {
            return Err(PipeError::MessageTooLarge {
                size: length,
                max: MAX_MESSAGE_SIZE,
            });
        }

        Ok(Self {
            magic,
            version,
            flags,
            length,
        })
    }
}