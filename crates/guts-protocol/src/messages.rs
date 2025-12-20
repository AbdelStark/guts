//! Protocol message definitions.

use crate::{ProtocolError, Result, Version, MAGIC, MAX_MESSAGE_SIZE};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use serde::{Deserialize, Serialize};

/// The kind of message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum MessageKind {
    /// Handshake initiation.
    Hello = 0,
    /// Handshake response.
    HelloAck = 1,
    /// Keep-alive ping.
    Ping = 2,
    /// Keep-alive pong.
    Pong = 3,
    /// Request repository refs.
    GetRefs = 10,
    /// Response with refs.
    Refs = 11,
    /// Request objects.
    GetObjects = 12,
    /// Response with objects.
    Objects = 13,
    /// Announce new commit.
    NewCommit = 20,
    /// Gossip message.
    Gossip = 30,
}

/// A protocol message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// The message kind.
    pub kind: MessageKind,
    /// The message payload.
    pub payload: Bytes,
}

impl Message {
    /// Creates a new message.
    #[must_use]
    pub fn new(kind: MessageKind, payload: impl Into<Bytes>) -> Self {
        Self {
            kind,
            payload: payload.into(),
        }
    }

    /// Creates a ping message.
    #[must_use]
    pub fn ping(nonce: u64) -> Self {
        Self::new(MessageKind::Ping, nonce.to_be_bytes().to_vec())
    }

    /// Creates a pong message.
    #[must_use]
    pub fn pong(nonce: u64) -> Self {
        Self::new(MessageKind::Pong, nonce.to_be_bytes().to_vec())
    }

    /// Encodes the message to bytes.
    ///
    /// Format:
    /// - 4 bytes: magic
    /// - 1 byte: message kind
    /// - 4 bytes: payload length (big-endian)
    /// - N bytes: payload
    #[must_use]
    pub fn encode(&self) -> Bytes {
        let mut buf = BytesMut::with_capacity(9 + self.payload.len());
        buf.put_slice(&MAGIC);
        buf.put_u8(self.kind as u8);
        buf.put_u32(self.payload.len() as u32);
        buf.put_slice(&self.payload);
        buf.freeze()
    }

    /// Decodes a message from bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if the message is malformed.
    pub fn decode(mut data: Bytes) -> Result<Self> {
        if data.len() < 9 {
            return Err(ProtocolError::Malformed("message too short".into()));
        }

        // Check magic
        let magic: [u8; 4] = data[..4].try_into().unwrap();
        if magic != MAGIC {
            return Err(ProtocolError::InvalidMagic);
        }
        data.advance(4);

        // Parse kind
        let kind_byte = data.get_u8();
        let kind = match kind_byte {
            0 => MessageKind::Hello,
            1 => MessageKind::HelloAck,
            2 => MessageKind::Ping,
            3 => MessageKind::Pong,
            10 => MessageKind::GetRefs,
            11 => MessageKind::Refs,
            12 => MessageKind::GetObjects,
            13 => MessageKind::Objects,
            20 => MessageKind::NewCommit,
            30 => MessageKind::Gossip,
            _ => return Err(ProtocolError::Malformed(format!("unknown message kind: {kind_byte}"))),
        };

        // Parse length
        let len = data.get_u32() as usize;
        if len > MAX_MESSAGE_SIZE {
            return Err(ProtocolError::TooLarge {
                size: len,
                max: MAX_MESSAGE_SIZE,
            });
        }

        if data.len() < len {
            return Err(ProtocolError::Malformed("incomplete payload".into()));
        }

        let payload = data.slice(..len);

        Ok(Self { kind, payload })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_roundtrip() {
        let msg = Message::ping(42);
        let encoded = msg.encode();
        let decoded = Message::decode(encoded).unwrap();

        assert_eq!(decoded.kind, MessageKind::Ping);
    }

    #[test]
    fn message_invalid_magic() {
        let data = Bytes::from_static(b"BAAD\x00\x00\x00\x00\x00");
        let result = Message::decode(data);
        assert!(matches!(result, Err(ProtocolError::InvalidMagic)));
    }
}
