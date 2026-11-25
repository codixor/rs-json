//! Shared parser limits and reader buffer sizing.

pub use rs_format::Limits;

/// Buffer sizing for reader-backed parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BufConfig {
    /// Refill buffer size for `io::Read` sources. bufjson uses 8 KiB; sweep
    /// 8/16/64/256 KiB in `benches/stream_large.rs` before fixing a default.
    pub read_buf: usize,
    /// Files at or below this size are read into one buffer and parsed with
    /// the slice lexer (fewest syscalls, best cache behaviour).
    pub small_file_max: u64,
    /// Minimum size for an anonymous mapped snapshot when a file exceeds `small_file_max`.
    pub mmap_min: u64,
}

impl Default for BufConfig {
    fn default() -> Self {
        Self {
            read_buf: 64 * 1024,
            small_file_max: 1024 * 1024,
            mmap_min: 8 * 1024 * 1024,
        }
    }
}
