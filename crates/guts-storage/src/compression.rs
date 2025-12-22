//! Compression utilities for storage.
//!
//! Provides configurable compression levels and statistics
//! for optimizing storage efficiency.

use std::sync::atomic::{AtomicU64, Ordering};

/// Compression level configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionLevel {
    /// No compression
    None,
    /// Fast compression (lower ratio)
    Fast,
    /// Default compression (balanced)
    Default,
    /// Best compression (slower, higher ratio)
    Best,
}

impl CompressionLevel {
    /// Converts to flate2 compression level.
    pub fn to_flate2(self) -> flate2::Compression {
        match self {
            CompressionLevel::None => flate2::Compression::none(),
            CompressionLevel::Fast => flate2::Compression::fast(),
            CompressionLevel::Default => flate2::Compression::default(),
            CompressionLevel::Best => flate2::Compression::best(),
        }
    }
}

impl Default for CompressionLevel {
    fn default() -> Self {
        CompressionLevel::Default
    }
}

/// Compression statistics for monitoring.
#[derive(Debug, Default)]
pub struct CompressionStats {
    /// Total bytes before compression.
    pub input_bytes: AtomicU64,
    /// Total bytes after compression.
    pub output_bytes: AtomicU64,
    /// Number of compression operations.
    pub compress_count: AtomicU64,
    /// Number of decompression operations.
    pub decompress_count: AtomicU64,
}

impl CompressionStats {
    /// Creates new compression stats.
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a compression operation.
    pub fn record_compress(&self, input_size: u64, output_size: u64) {
        self.input_bytes.fetch_add(input_size, Ordering::Relaxed);
        self.output_bytes.fetch_add(output_size, Ordering::Relaxed);
        self.compress_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Records a decompression operation.
    pub fn record_decompress(&self) {
        self.decompress_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Returns the compression ratio (output/input).
    pub fn compression_ratio(&self) -> f64 {
        let input = self.input_bytes.load(Ordering::Relaxed);
        let output = self.output_bytes.load(Ordering::Relaxed);
        if input == 0 {
            1.0
        } else {
            output as f64 / input as f64
        }
    }

    /// Returns the space savings percentage.
    pub fn space_savings(&self) -> f64 {
        1.0 - self.compression_ratio()
    }

    /// Returns a snapshot of the stats.
    pub fn snapshot(&self) -> CompressionStatsSnapshot {
        CompressionStatsSnapshot {
            input_bytes: self.input_bytes.load(Ordering::Relaxed),
            output_bytes: self.output_bytes.load(Ordering::Relaxed),
            compress_count: self.compress_count.load(Ordering::Relaxed),
            decompress_count: self.decompress_count.load(Ordering::Relaxed),
        }
    }
}

/// Snapshot of compression statistics.
#[derive(Debug, Clone)]
pub struct CompressionStatsSnapshot {
    pub input_bytes: u64,
    pub output_bytes: u64,
    pub compress_count: u64,
    pub decompress_count: u64,
}

impl CompressionStatsSnapshot {
    /// Returns the compression ratio.
    pub fn compression_ratio(&self) -> f64 {
        if self.input_bytes == 0 {
            1.0
        } else {
            self.output_bytes as f64 / self.input_bytes as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_level_default() {
        let level = CompressionLevel::default();
        assert_eq!(level, CompressionLevel::Default);
    }

    #[test]
    fn test_compression_stats() {
        let stats = CompressionStats::new();

        stats.record_compress(1000, 500);
        stats.record_compress(1000, 500);
        stats.record_decompress();

        let snapshot = stats.snapshot();
        assert_eq!(snapshot.input_bytes, 2000);
        assert_eq!(snapshot.output_bytes, 1000);
        assert_eq!(snapshot.compress_count, 2);
        assert_eq!(snapshot.decompress_count, 1);
        assert!((snapshot.compression_ratio() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_space_savings() {
        let stats = CompressionStats::new();
        stats.record_compress(1000, 250);

        assert!((stats.space_savings() - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_compression_ratio_zero_input() {
        let stats = CompressionStats::new();
        assert_eq!(stats.compression_ratio(), 1.0);
    }
}
