//! Archive generation for repository downloads.

use bytes::{BufMut, BytesMut};
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::Write;

use crate::error::{CompatError, Result};

/// Archive format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveFormat {
    /// Gzipped tar archive.
    TarGz,
    /// Zip archive.
    Zip,
}

impl ArchiveFormat {
    /// Get the content type for this format.
    pub fn content_type(&self) -> &'static str {
        match self {
            Self::TarGz => "application/gzip",
            Self::Zip => "application/zip",
        }
    }

    /// Get the file extension for this format.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::TarGz => ".tar.gz",
            Self::Zip => ".zip",
        }
    }

    /// Get the Content-Disposition filename.
    pub fn filename(&self, repo_name: &str, ref_name: &str) -> String {
        // Sanitize ref name for filename
        let safe_ref = ref_name
            .replace('/', "-")
            .replace('\\', "-")
            .replace(':', "-");
        format!("{}-{}{}", repo_name, safe_ref, self.extension())
    }
}

/// A file entry to include in an archive.
#[derive(Debug, Clone)]
pub struct ArchiveEntry {
    /// Path within the archive (relative).
    pub path: String,
    /// File content.
    pub content: Vec<u8>,
    /// Unix file mode (e.g., 0o644 for regular file, 0o755 for executable).
    pub mode: u32,
    /// Whether this is an executable file.
    pub executable: bool,
}

impl ArchiveEntry {
    /// Create a new regular file entry.
    pub fn file(path: String, content: Vec<u8>) -> Self {
        Self {
            path,
            content,
            mode: 0o644,
            executable: false,
        }
    }

    /// Create a new executable file entry.
    pub fn executable(path: String, content: Vec<u8>) -> Self {
        Self {
            path,
            content,
            mode: 0o755,
            executable: true,
        }
    }
}

/// Builder for creating tar.gz archives.
pub struct TarGzBuilder {
    entries: Vec<ArchiveEntry>,
    prefix: String,
}

impl TarGzBuilder {
    /// Create a new tar.gz builder with a prefix directory.
    pub fn new(prefix: String) -> Self {
        Self {
            entries: Vec::new(),
            prefix,
        }
    }

    /// Add an entry to the archive.
    pub fn add(&mut self, entry: ArchiveEntry) {
        self.entries.push(entry);
    }

    /// Build the archive and return the bytes.
    pub fn build(self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        let encoder = GzEncoder::new(&mut buffer, Compression::default());
        let mut tar = tar::Builder::new(encoder);

        for entry in self.entries {
            let path = if self.prefix.is_empty() {
                entry.path
            } else {
                format!("{}/{}", self.prefix, entry.path)
            };

            let mut header = tar::Header::new_gnu();
            header.set_size(entry.content.len() as u64);
            header.set_mode(entry.mode);
            header.set_mtime(0); // Use 0 for reproducible builds
            header.set_cksum();

            tar.append_data(&mut header, &path, entry.content.as_slice())
                .map_err(|e| CompatError::ArchiveFailed(e.to_string()))?;
        }

        tar.into_inner()
            .map_err(|e| CompatError::ArchiveFailed(e.to_string()))?
            .finish()
            .map_err(|e| CompatError::ArchiveFailed(e.to_string()))?;

        Ok(buffer)
    }
}

/// Builder for creating zip archives.
pub struct ZipBuilder {
    entries: Vec<ArchiveEntry>,
    prefix: String,
}

impl ZipBuilder {
    /// Create a new zip builder with a prefix directory.
    pub fn new(prefix: String) -> Self {
        Self {
            entries: Vec::new(),
            prefix,
        }
    }

    /// Add an entry to the archive.
    pub fn add(&mut self, entry: ArchiveEntry) {
        self.entries.push(entry);
    }

    /// Build the archive and return the bytes.
    pub fn build(self) -> Result<Vec<u8>> {
        use std::io::Cursor;
        use zip::write::SimpleFileOptions;
        use zip::ZipWriter;

        let mut buffer = Cursor::new(Vec::new());
        let mut zip = ZipWriter::new(&mut buffer);

        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o644);

        for entry in self.entries {
            let path = if self.prefix.is_empty() {
                entry.path
            } else {
                format!("{}/{}", self.prefix, entry.path)
            };

            let file_options = if entry.executable {
                options.unix_permissions(0o755)
            } else {
                options
            };

            zip.start_file(&path, file_options)
                .map_err(|e| CompatError::ArchiveFailed(e.to_string()))?;
            zip.write_all(&entry.content)
                .map_err(|e| CompatError::ArchiveFailed(e.to_string()))?;
        }

        zip.finish()
            .map_err(|e| CompatError::ArchiveFailed(e.to_string()))?;

        Ok(buffer.into_inner())
    }
}

/// Create an archive from entries.
pub fn create_archive(
    format: ArchiveFormat,
    prefix: String,
    entries: Vec<ArchiveEntry>,
) -> Result<Vec<u8>> {
    match format {
        ArchiveFormat::TarGz => {
            let mut builder = TarGzBuilder::new(prefix);
            for entry in entries {
                builder.add(entry);
            }
            builder.build()
        }
        ArchiveFormat::Zip => {
            let mut builder = ZipBuilder::new(prefix);
            for entry in entries {
                builder.add(entry);
            }
            builder.build()
        }
    }
}

/// Streaming archive builder for large repositories.
///
/// This allows building archives incrementally to avoid loading
/// all files into memory at once.
pub struct StreamingArchive {
    buffer: BytesMut,
    format: ArchiveFormat,
}

impl StreamingArchive {
    /// Create a new streaming archive builder.
    pub fn new(format: ArchiveFormat, capacity: usize) -> Self {
        Self {
            buffer: BytesMut::with_capacity(capacity),
            format,
        }
    }

    /// Get the format of this archive.
    pub fn format(&self) -> ArchiveFormat {
        self.format
    }

    /// Append raw bytes to the buffer.
    pub fn append(&mut self, data: &[u8]) {
        self.buffer.put_slice(data);
    }

    /// Get the current size.
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Take the buffer contents.
    pub fn take(self) -> Vec<u8> {
        self.buffer.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archive_format() {
        assert_eq!(ArchiveFormat::TarGz.content_type(), "application/gzip");
        assert_eq!(ArchiveFormat::Zip.content_type(), "application/zip");
        assert_eq!(ArchiveFormat::TarGz.extension(), ".tar.gz");
        assert_eq!(ArchiveFormat::Zip.extension(), ".zip");
    }

    #[test]
    fn test_archive_filename() {
        assert_eq!(
            ArchiveFormat::TarGz.filename("repo", "v1.0.0"),
            "repo-v1.0.0.tar.gz"
        );
        assert_eq!(
            ArchiveFormat::Zip.filename("repo", "feature/test"),
            "repo-feature-test.zip"
        );
    }

    #[test]
    fn test_tar_gz_archive() {
        let entries = vec![
            ArchiveEntry::file("file1.txt".to_string(), b"Hello".to_vec()),
            ArchiveEntry::file("dir/file2.txt".to_string(), b"World".to_vec()),
        ];

        let archive = create_archive(ArchiveFormat::TarGz, "test-repo".to_string(), entries);
        assert!(archive.is_ok());

        let bytes = archive.unwrap();
        assert!(!bytes.is_empty());
        // Check gzip magic bytes
        assert_eq!(bytes[0], 0x1f);
        assert_eq!(bytes[1], 0x8b);
    }

    #[test]
    fn test_zip_archive() {
        let entries = vec![
            ArchiveEntry::file("file1.txt".to_string(), b"Hello".to_vec()),
            ArchiveEntry::executable("script.sh".to_string(), b"#!/bin/bash".to_vec()),
        ];

        let archive = create_archive(ArchiveFormat::Zip, "test-repo".to_string(), entries);
        assert!(archive.is_ok());

        let bytes = archive.unwrap();
        assert!(!bytes.is_empty());
        // Check zip magic bytes
        assert_eq!(bytes[0], 0x50);
        assert_eq!(bytes[1], 0x4b);
    }

    #[test]
    fn test_archive_entry() {
        let entry = ArchiveEntry::file("test.txt".to_string(), b"content".to_vec());
        assert_eq!(entry.mode, 0o644);
        assert!(!entry.executable);

        let exec = ArchiveEntry::executable("run.sh".to_string(), b"#!/bin/sh".to_vec());
        assert_eq!(exec.mode, 0o755);
        assert!(exec.executable);
    }
}
