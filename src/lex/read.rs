//! Buffered lexer over `std::io::Read` for large files and sockets.
//!
//! Design (adapted from bufjson `ReadAnalyzer`, default buffer 8 KiB there):
//! * One `Box<[u8]>` buffer of `BufConfig::read_buf` bytes, refilled with
//!   `read` (never `read_exact`), `ErrorKind::Interrupted` retried.
//! * Tokens are scanned with the same `scan` kernels as `SliceLexer`.
//! * A token crossing the buffer end is moved into `spill: Vec<u8>` (reused,
//!   capped by `Limits::max_token_bytes`); `content()` then points into spill.
//! * Newlines are counted per consumed chunk (`memchr`-style count) so error
//!   positions keep line/col without per-byte tracking.
//! * `into_inner()` returns the reader and unconsumed bytes (for NDJSON and
//!   for concatenated-JSON protocols).

use super::{Lexer, Span, Token};
use crate::config::{BufConfig, Limits};
use crate::Error;
use alloc::{boxed::Box, vec::Vec};
use std::io::Read;

/// Streaming lexer with bounded memory.
#[derive(Debug)]
pub struct ReadLexer<R: Read> {
    reader: R,
    buf: Box<[u8]>,
    head: usize,
    tail: usize,
    spill: Vec<u8>,
    in_spill: bool,
    consumed: u64,
    line: u32,
    span: Span,
    limits: Limits,
}

impl<R: Read> ReadLexer<R> {
    /// New lexer with explicit buffer config.
    pub fn new(reader: R, cfg: BufConfig, limits: Limits) -> Self {
        Self {
            reader,
            buf: alloc::vec![0; cfg.read_buf.max(512)].into_boxed_slice(),
            head: 0,
            tail: 0,
            spill: Vec::new(),
            in_spill: false,
            consumed: 0,
            line: 1,
            span: Span::default(),
            limits,
        }
    }

    /// Recover the reader plus any unread buffered bytes.
    pub fn into_inner(self) -> (R, Vec<u8>) {
        let rest = self.buf.get(self.head..self.tail).unwrap_or_default().to_vec();
        (self.reader, rest)
    }

    #[allow(dead_code)]
    fn refill(&mut self) -> std::io::Result<usize> {
        let _ = (&self.consumed, &self.line, &self.limits);
        todo!("agent: compact [head..tail] to front, read into the free tail, retry Interrupted")
    }
}

impl<R: Read> Lexer for ReadLexer<R> {
    fn next_token(&mut self) -> Result<Token, Error> {
        todo!("agent: same token grammar as SliceLexer; spill on boundary")
    }

    fn span(&self) -> Span {
        self.span
    }

    fn content(&self) -> &[u8] {
        if self.in_spill {
            &self.spill
        } else {
            let s = usize::try_from(self.span.start - self.consumed).unwrap_or(0);
            self.buf.get(s..s + self.span.len as usize).unwrap_or_default()
        }
    }
}
