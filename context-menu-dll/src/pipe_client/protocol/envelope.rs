//! Message envelope: header + payload.

use super::header::*;
use crate::pipe_client::error::PipeError;

#[derive(Debug, Clone)]
pub struct MessageEnvelope {
    pub header: MessageHeader,
    pub payload: Vec<u8>,
}

impl MessageEnvelope {
    pub fn new(payload: Vec<u8>) -> Result<Self, PipeError> {
        if payload.len() as u32 > MAX_MESSAGE_SIZE {
            return Err(PipeError::MessageTooLarge {
                size: payload.len() as u32,
                max: MAX_MESSAGE_SIZE,
            });
        }
        let header = MessageHeader::new(payload.len() as u32);
        Ok(Self { header, payload })
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(MessageHeader::SIZE + self.payload.len());
        result.extend_from_slice(&self.header.encode());
        result.extend_from_slice(&self.payload);
        result
    }

    pub fn decode(data: &[u8]) -> Result<Self, PipeError> {
        if data.len() < MessageHeader::SIZE {
            return Err(PipeError::IncompleteRead {
                expected: MessageHeader::SIZE as u32,
                actual: data.len() as u32,
            });
        }

        let header = MessageHeader::decode(&data[..MessageHeader::SIZE])?;
        let expected_len = header.length as usize;

        if data.len() < MessageHeader::SIZE + expected_len {
            return Err(PipeError::IncompleteRead {
                expected: (MessageHeader::SIZE + expected_len) as u32,
                actual: data.len() as u32,
            });
        }

        let payload = data[MessageHeader::SIZE..MessageHeader::SIZE + expected_len].to_vec();

        Ok(Self { header, payload })
    }
}