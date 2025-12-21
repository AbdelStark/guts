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

    #[test]
    fn test_pktline_response_end() {
        assert_eq!(PktLine::ResponseEnd.encode(), b"0002");
    }

    #[test]
    fn test_pktline_from_bytes() {
        let pkt = PktLine::from_bytes(b"test data".to_vec());
        assert_eq!(pkt.data(), Some(b"test data".as_slice()));
    }

    #[test]
    fn test_pktline_is_flush() {
        assert!(PktLine::Flush.is_flush());
        assert!(!PktLine::from_string("test").is_flush());
        assert!(!PktLine::Delimiter.is_flush());
        assert!(!PktLine::ResponseEnd.is_flush());
    }

    #[test]
    fn test_pktline_data() {
        let pkt = PktLine::from_string("hello");
        assert_eq!(pkt.data(), Some(b"hello".as_slice()));

        assert!(PktLine::Flush.data().is_none());
        assert!(PktLine::Delimiter.data().is_none());
        assert!(PktLine::ResponseEnd.data().is_none());
    }

    #[test]
    fn test_pktline_as_str() {
        let pkt = PktLine::from_string("hello\n");
        assert_eq!(pkt.as_str(), Some("hello"));

        let pkt2 = PktLine::from_string("no newline");
        assert_eq!(pkt2.as_str(), Some("no newline"));
    }

    #[test]
    fn test_pktline_as_str_invalid_utf8() {
        let pkt = PktLine::from_bytes(vec![0xff, 0xfe]);
        assert!(pkt.as_str().is_none());
    }

    #[test]
    fn test_pktline_reader_eof() {
        let reader = PktLineReader::new(Cursor::new(Vec::<u8>::new()));
        let result = reader.into_inner();
        assert_eq!(result.position(), 0);
    }

    #[test]
    fn test_pktline_read_until_flush() {
        let mut buf = Vec::new();
        {
            let mut writer = PktLineWriter::new(&mut buf);
            writer.write_line("line1").unwrap();
            writer.write_line("line2").unwrap();
            writer.flush_pkt().unwrap();
            writer.write_line("line3").unwrap();
        }

        let mut reader = PktLineReader::new(Cursor::new(buf));
        let packets = reader.read_until_flush().unwrap();
        assert_eq!(packets.len(), 2);
    }

    #[test]
    fn test_pktline_writer_write_line() {
        let mut buf = Vec::new();
        {
            let mut writer = PktLineWriter::new(&mut buf);
            writer.write_line("test").unwrap();
        }
        // "test\n" is 5 bytes, + 4 for length = 9, so hex "0009"
        assert!(buf.starts_with(b"0009"));
        assert!(buf.ends_with(b"test\n"));
    }

    #[test]
    fn test_pktline_writer_write_line_with_newline() {
        let mut buf = Vec::new();
        {
            let mut writer = PktLineWriter::new(&mut buf);
            writer.write_line("test\n").unwrap();
        }
        // Should not double the newline
        assert!(buf.ends_with(b"test\n"));
        assert!(!buf.ends_with(b"test\n\n"));
    }

    #[test]
    fn test_pktline_writer_write_data() {
        let mut buf = Vec::new();
        {
            let mut writer = PktLineWriter::new(&mut buf);
            writer.write_data(b"binary\x00data").unwrap();
        }
        assert!(buf.len() > 4); // At least the length prefix
    }

    #[test]
    fn test_pktline_writer_flush() {
        let mut buf = Vec::new();
        {
            let mut writer = PktLineWriter::new(&mut buf);
            writer.write_line("test").unwrap();
            writer.flush().unwrap();
        }
        // Should have been flushed to the buffer
        assert!(!buf.is_empty());
    }

    #[test]
    fn test_pktline_writer_into_inner() {
        let buf = Vec::new();
        let writer = PktLineWriter::new(buf);
        let inner = writer.into_inner();
        assert!(inner.is_empty());
    }

    #[test]
    fn test_pktline_reader_inner_mut() {
        let cursor = Cursor::new(Vec::<u8>::new());
        let mut reader = PktLineReader::new(cursor);
        let inner = reader.inner_mut();
        assert_eq!(inner.position(), 0);
    }

    #[test]
    fn test_pktline_read_delimiter() {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"0001");

        let mut reader = PktLineReader::new(Cursor::new(buf));
        assert_eq!(reader.read().unwrap(), Some(PktLine::Delimiter));
    }

    #[test]
    fn test_pktline_read_response_end() {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"0002");

        let mut reader = PktLineReader::new(Cursor::new(buf));
        assert_eq!(reader.read().unwrap(), Some(PktLine::ResponseEnd));
    }

    #[test]
    fn test_pktline_equality() {
        assert_eq!(PktLine::Flush, PktLine::Flush);
        assert_eq!(PktLine::Delimiter, PktLine::Delimiter);
        assert_eq!(PktLine::ResponseEnd, PktLine::ResponseEnd);
        assert_eq!(PktLine::from_string("test"), PktLine::from_string("test"));
        assert_ne!(PktLine::Flush, PktLine::Delimiter);
    }

    #[test]
    fn test_pktline_clone() {
        let pkt = PktLine::from_string("test");
        let cloned = pkt.clone();
        assert_eq!(pkt, cloned);
    }

    #[test]
    fn test_pktline_debug() {
        let pkt = PktLine::Flush;
        let debug = format!("{:?}", pkt);
        assert!(debug.contains("Flush"));
    }

    #[test]
    fn test_pktline_read_invalid_length() {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"0003"); // Invalid: 3 is less than 4

        let mut reader = PktLineReader::new(Cursor::new(buf));
        let result = reader.read();
        assert!(result.is_err());
    }

    #[test]
    fn test_pktline_large_packet() {
        let data = "x".repeat(1000);
        let pkt = PktLine::from_string(&data);
        let encoded = pkt.encode();

        // Verify we can read it back
        let mut reader = PktLineReader::new(Cursor::new(encoded));
        let read_pkt = reader.read().unwrap().unwrap();
        assert_eq!(read_pkt.data().unwrap().len(), 1000);
    }

    #[test]
    fn test_pktline_empty_data() {
        let pkt = PktLine::from_bytes(Vec::new());
        let encoded = pkt.encode();
        assert_eq!(&encoded[..4], b"0004"); // Just the length prefix
    }

    #[test]
    fn test_pktline_read_eof_on_empty() {
        let mut reader = PktLineReader::new(Cursor::new(Vec::<u8>::new()));
        let result = reader.read().unwrap();
        assert!(result.is_none());
    }
}
