//! Git pkt-line format implementation.
//!
//! The pkt-line format is used for all git protocol communication.
//! Each line is prefixed with a 4-character hex length, or "0000" for flush.

use crate::{GitError, Result};
use std::io::{Read, Write};

/// A pkt-line packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PktLine {
    /// Data line with content.
    Data(Vec<u8>),
    /// Flush packet (0000).
    Flush,
    /// Delimiter packet (0001).
    Delimiter,
    /// Response-end packet (0002).
    ResponseEnd,
}

impl PktLine {
    /// Creates a data packet from a string slice.
    pub fn from_string(s: &str) -> Self {
        Self::Data(s.as_bytes().to_vec())
    }

    /// Creates a data packet from bytes.
    pub fn from_bytes(b: impl Into<Vec<u8>>) -> Self {
        Self::Data(b.into())
    }

    /// Encodes the packet to bytes.
    pub fn encode(&self) -> Vec<u8> {
        match self {
            Self::Data(data) => {
                let len = data.len() + 4; // 4 bytes for the length prefix
                let mut result = format!("{:04x}", len).into_bytes();
                result.extend_from_slice(data);
                result
            }
            Self::Flush => b"0000".to_vec(),
            Self::Delimiter => b"0001".to_vec(),
            Self::ResponseEnd => b"0002".to_vec(),
        }
    }

    /// Returns true if this is a flush packet.
    pub fn is_flush(&self) -> bool {
        matches!(self, Self::Flush)
    }

    /// Returns the data content, or None for special packets.
    pub fn data(&self) -> Option<&[u8]> {
        match self {
            Self::Data(data) => Some(data),
            _ => None,
        }
    }

    /// Returns the data as a string, trimming any trailing newline.
    pub fn as_str(&self) -> Option<&str> {
        self.data()
            .and_then(|d| std::str::from_utf8(d).ok())
            .map(|s| s.trim_end_matches('\n'))
    }
}

/// Reader for pkt-line format.
pub struct PktLineReader<R> {
    reader: R,
}

impl<R: Read> PktLineReader<R> {
    /// Creates a new pkt-line reader.
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    /// Reads the next packet.
    pub fn read(&mut self) -> Result<Option<PktLine>> {
        let mut len_buf = [0u8; 4];
        match self.reader.read_exact(&mut len_buf) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(e.into()),
        }

        let len_str = std::str::from_utf8(&len_buf)
            .map_err(|_| GitError::InvalidPktLine("invalid length prefix".to_string()))?;

        match len_str {
            "0000" => Ok(Some(PktLine::Flush)),
            "0001" => Ok(Some(PktLine::Delimiter)),
            "0002" => Ok(Some(PktLine::ResponseEnd)),
            _ => {
                let len = u16::from_str_radix(len_str, 16)
                    .map_err(|_| GitError::InvalidPktLine("invalid length".to_string()))?
                    as usize;

                if len < 4 {
                    return Err(GitError::InvalidPktLine("length too small".to_string()));
                }

                let data_len = len - 4;
                let mut data = vec![0u8; data_len];
                self.reader.read_exact(&mut data)?;

                Ok(Some(PktLine::Data(data)))
            }
        }
    }

    /// Reads all packets until a flush packet.
    pub fn read_until_flush(&mut self) -> Result<Vec<PktLine>> {
        let mut packets = Vec::new();
        loop {
            match self.read()? {
                Some(PktLine::Flush) | None => break,
                Some(pkt) => packets.push(pkt),
            }
        }
        Ok(packets)
    }

    /// Returns a mutable reference to the inner reader.
    pub fn inner_mut(&mut self) -> &mut R {
        &mut self.reader
    }

    /// Consumes the reader and returns the inner reader.
    pub fn into_inner(self) -> R {
        self.reader
    }
}

/// Writer for pkt-line format.
pub struct PktLineWriter<W> {
    writer: W,
}

impl<W: Write> PktLineWriter<W> {
    /// Creates a new pkt-line writer.
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    /// Writes a packet.
    pub fn write(&mut self, pkt: &PktLine) -> Result<()> {
        self.writer.write_all(&pkt.encode())?;
        Ok(())
    }

    /// Writes a data line.
    pub fn write_data(&mut self, data: &[u8]) -> Result<()> {
        self.write(&PktLine::Data(data.to_vec()))
    }

    /// Writes a string line (with newline).
    pub fn write_line(&mut self, s: &str) -> Result<()> {
        let mut data = s.as_bytes().to_vec();
        if !s.ends_with('\n') {
            data.push(b'\n');
        }
        self.write(&PktLine::Data(data))
    }

    /// Writes a flush packet.
    pub fn flush_pkt(&mut self) -> Result<()> {
        self.write(&PktLine::Flush)
    }

    /// Flushes the underlying writer.
    pub fn flush(&mut self) -> Result<()> {
        self.writer.flush()?;
        Ok(())
    }

    /// Returns the inner writer.
    pub fn into_inner(self) -> W {
        self.writer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_pktline_encode() {
        assert_eq!(PktLine::from_string("hello\n").encode(), b"000ahello\n");
        assert_eq!(PktLine::Flush.encode(), b"0000");
        assert_eq!(PktLine::Delimiter.encode(), b"0001");
    }

    #[test]
    fn test_pktline_roundtrip() {
        let packets = vec![
            PktLine::from_string("hello\n"),
            PktLine::from_string("world\n"),
            PktLine::Flush,
        ];

        let mut buf = Vec::new();
        {
            let mut writer = PktLineWriter::new(&mut buf);
            for pkt in &packets {
                writer.write(pkt).unwrap();
            }
        }

        let mut reader = PktLineReader::new(Cursor::new(buf));
        assert_eq!(reader.read().unwrap(), Some(packets[0].clone()));
        assert_eq!(reader.read().unwrap(), Some(packets[1].clone()));
        assert_eq!(reader.read().unwrap(), Some(PktLine::Flush));
    }
}
