//! Parser limits and tuning knobs. All defaults are provisional until the
//! benchmark gates in `docs/PLAN.md` confirm them.

/// Hard limits that protect against hostile input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// Maximum container nesting. The parser is iterative, so this is a
    /// policy limit, not a stack-safety one. Default 512 (serde_json uses 128).
    pub max_depth: u32,
    /// Maximum input size for the tape DOM (tape offsets are `u32`).
    /// Larger inputs must use the streaming layer.
    pub max_document_bytes: u64,
    /// Maximum length of a single string or number token in the streaming
    /// layer (bounds the spill buffer when a token straddles refills).
    pub max_token_bytes: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_depth: 512,
            max_document_bytes: u64::from(u32::MAX),
            max_token_bytes: 64 * 1024 * 1024,
        }
    }
}

/// Buffer sizing for reader-backed parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BufConfig {
    /// Refill buffer size for `io::Read` sources. bufjson uses 8 KiB; sweep
    /// 8/16/64/256 KiB in `benches/stream_large.rs` before fixing a default.
    pub read_buf: usize,
    /// Files at or below this size are read into one buffer and parsed with
    /// the slice lexer (fewest syscalls, best cache behaviour).
    pub small_file_max: u64,
    /// Files above this size use mmap (feature `mmap`) for DOM parsing.
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
