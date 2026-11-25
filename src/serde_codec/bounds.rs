//! Count source bytes before handing them to the typed codec. This tracks JSON
//! token boundaries only; Serde retains responsibility for grammar, data errors
//! and its type-specific handling of string payloads.
use super::{JsonSerdeError, JsonSerdeResult};
use crate::{Limits, Pos};
use std::io::{self, Read};

// The typed backend uses the call stack. Check this ceiling on encoded input
// before passing an opener to Serde, even when the requested limit is higher.
pub(super) const MAX_DEPTH: u32 = 128;

#[derive(Clone, Copy)]
enum Body {
    Outside,
    Atom,
    String,
    Escape,
}

struct Budget {
    limits: Limits,
    depth: u32,
    body: Body,
    token_bytes: usize,
    pos: Pos,
}

impl Budget {
    fn new(limits: Limits) -> Self {
        Self {
            limits: Limits {
                max_depth: limits.max_depth.min(MAX_DEPTH),
                ..limits
            },
            depth: 0,
            body: Body::Outside,
            token_bytes: 0,
            pos: Pos {
                offset: 0,
                line: 1,
                col: 1,
            },
        }
    }

    fn charge_token(&mut self) -> Result<(), Pos> {
        if self.token_bytes >= self.limits.max_token_bytes {
            return Err(self.pos);
        }
        self.token_bytes += 1;
        Ok(())
    }

    fn check(&mut self, bytes: &[u8]) -> Result<(), Pos> {
        for &byte in bytes {
            if self.pos.offset >= self.limits.max_document_bytes {
                return Err(self.pos);
            }
            match (self.body, byte) {
                (Body::String, b'"') => self.body = Body::Outside,
                (Body::String, b'\\') => {
                    self.charge_token()?;
                    self.body = Body::Escape;
                }
                (Body::String | Body::Escape, _) => {
                    self.charge_token()?;
                    self.body = Body::String;
                }
                (_, b'"') => {
                    self.token_bytes = 0;
                    self.body = Body::String;
                }
                (_, b'[' | b'{') => {
                    if self.depth >= self.limits.max_depth {
                        return Err(self.pos);
                    }
                    self.depth += 1;
                    self.body = Body::Outside;
                }
                (_, b']' | b'}') => {
                    self.depth = self.depth.saturating_sub(1);
                    self.body = Body::Outside;
                }
                (_, b':' | b',' | b' ' | b'\n' | b'\r' | b'\t') => self.body = Body::Outside,
                (_, _) => {
                    if !matches!(self.body, Body::Atom) {
                        self.token_bytes = 0;
                        self.body = Body::Atom;
                    }
                    self.charge_token()?;
                }
            }
            self.pos.offset += 1;
            if byte == b'\n' {
                self.pos.line = self.pos.line.saturating_add(1);
                self.pos.col = 1;
            } else {
                self.pos.col = self.pos.col.saturating_add(1);
            }
        }
        Ok(())
    }
}

pub(super) fn check_slice(bytes: &[u8], limits: Limits) -> JsonSerdeResult<()> {
    // One byte opens at most one container or contributes one token byte.
    if bytes.len() as u64 <= limits.max_document_bytes
        && bytes.len() <= limits.max_token_bytes
        && bytes.len() as u64 <= u64::from(limits.max_depth.min(MAX_DEPTH))
    {
        return Ok(());
    }
    Budget::new(limits)
        .check(bytes)
        .map_err(|pos| JsonSerdeError::Limit { pos })
}

pub(super) struct BoundedRead<R> {
    reader: R,
    budget: Budget,
    failure: Option<Pos>,
}

impl<R: Read> BoundedRead<R> {
    pub(super) fn new(reader: R, limits: Limits) -> Self {
        Self {
            reader,
            budget: Budget::new(limits),
            failure: None,
        }
    }

    pub(super) fn limit_position(&self) -> Option<Pos> {
        self.failure
    }
}

impl<R: Read> Read for BoundedRead<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.failure.is_some() {
            return Err(io::ErrorKind::InvalidData.into());
        }
        let n = self.reader.read(buf)?;
        if let Err(pos) = self
            .budget
            .check(buf.get(..n).ok_or(io::ErrorKind::InvalidData)?)
        {
            self.failure = Some(pos);
            return Err(io::ErrorKind::InvalidData.into());
        }
        Ok(n)
    }
}
