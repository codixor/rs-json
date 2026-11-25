//! Resume tokens that cannot be completed from a contiguous refill.
use super::ReadLexer;
use crate::{
    Error, ErrorKind,
    unescape::{EscapeError, escape_prefix},
};
use std::io::Read;

pub(super) fn literal(input: &mut ReadLexer<impl Read>, mut word: &[u8]) -> Result<(), Error> {
    let start = input.offset();
    while !word.is_empty() {
        let at_limit = input.offset() == input.limit_end.min(input.document_end);
        let buf = input.chunk()?;
        let n = word.len().min(buf.len());
        let (prefix, rest) = word.split_at(n);
        if n == 0 {
            return Err(input.error(input.offset(), ErrorKind::UnexpectedEof));
        }
        if at_limit {
            return Err(input.error(input.offset(), ErrorKind::SizeLimit));
        }
        if !buf.starts_with(prefix) {
            return Err(input.error(start, ErrorKind::InvalidToken));
        }
        input.advance(n)?;
        word = rest;
    }
    Ok(())
}

fn digits(input: &mut ReadLexer<impl Read>) -> Result<(), Error> {
    let start = input.offset();
    loop {
        let buf = input.chunk()?;
        let n = buf.iter().take_while(|b| b.is_ascii_digit()).count();
        let done = n < buf.len() || n == 0;
        input.advance(n)?;
        if done {
            break;
        }
    }
    if start == input.offset() {
        let kind = if input.peek()?.is_none() {
            ErrorKind::UnexpectedEof
        } else {
            ErrorKind::InvalidNumber
        };
        Err(input.error(start, kind))
    } else {
        Ok(())
    }
}

pub(super) fn number(input: &mut ReadLexer<impl Read>) -> Result<(), Error> {
    if input.peek()? == Some(b'-') {
        input.advance(1)?;
    }
    if input.peek()? == Some(b'0') {
        input.advance(1)?;
        if matches!(input.peek()?, Some(b'0'..=b'9')) {
            return Err(input.error(input.offset(), ErrorKind::InvalidNumber));
        }
    } else {
        digits(input)?;
    }
    if input.peek()? == Some(b'.') {
        input.advance(1)?;
        digits(input)?;
    }
    if matches!(input.peek()?, Some(b'e' | b'E')) {
        input.advance(1)?;
        if matches!(input.peek()?, Some(b'+' | b'-')) {
            input.advance(1)?;
        }
        digits(input)?;
    }
    Ok(())
}

pub(super) fn string_body(input: &mut ReadLexer<impl Read>) -> Result<bool, Error> {
    let mut escaped = false;
    loop {
        let buf = input.chunk()?;
        if buf.is_empty() {
            return Err(input.error(input.offset(), ErrorKind::UnexpectedEof));
        }
        let n = crate::scan::string_run_end(buf, 0);
        let stop = buf.get(n).copied();
        input.advance(n)?;
        match stop {
            Some(b'"') => return Ok(escaped),
            Some(b'\\') => {
                string_escape(input)?;
                escaped = true;
            }
            Some(0x80..=0xff) => non_ascii(input)?,
            Some(_) => return Err(input.error(input.offset(), ErrorKind::InvalidString)),
            None => {} // plain bytes occupied the rest of this chunk
        }
    }
}

fn string_escape(input: &mut ReadLexer<impl Read>) -> Result<(), Error> {
    let start = input.offset();
    input.advance(1)?;
    let buf = input.chunk()?;
    let mut scratch = [0u8; 11]; // uXXXX backslash uXXXX
    let mut filled = match escape_prefix(buf) {
        Ok((_, n)) => return input.advance(n),
        Err(EscapeError::Invalid(kind)) => return Err(input.error(start, kind)),
        Err(EscapeError::Incomplete(kind)) if buf.is_empty() => {
            return Err(input.error(start, kind));
        }
        Err(EscapeError::Incomplete(_)) => {
            scratch
                .get_mut(..buf.len())
                .expect("an incomplete escape fits in 11 bytes")
                .copy_from_slice(buf);
            buf.len()
        }
    };
    input.advance(filled)?;
    // Only an escape split by a refill takes this path. Never read past its last byte.
    loop {
        match escape_prefix(scratch.get(..filled).expect("filled escape prefix")) {
            Ok(_) => return Ok(()),
            Err(EscapeError::Invalid(kind)) => return Err(input.error(start, kind)),
            Err(EscapeError::Incomplete(kind)) => {
                let Some(byte) = input.peek()? else {
                    return Err(input.error(start, kind));
                };
                *scratch.get_mut(filled).expect("valid escape width") = byte;
                filled += 1;
                input.advance(1)?;
            }
        }
    }
}

fn non_ascii(input: &mut ReadLexer<impl Read>) -> Result<(), Error> {
    let start = input.offset();
    loop {
        let buf = input.chunk()?;
        let n = buf.iter().position(u8::is_ascii).unwrap_or(buf.len());
        let run = buf.get(..n).expect("prefix of the current buffer");
        if n == 0 {
            return Ok(());
        }
        if simdutf8::basic::from_utf8(run).is_ok() {
            let done = n < buf.len();
            input.advance(n)?;
            if done {
                return Ok(());
            }
            continue;
        }

        if n < buf.len() {
            return Err(input.error(start, ErrorKind::InvalidString));
        }
        // A multibyte code point may end in the next buffer. Keep at most
        // four bytes, leaving the next ASCII delimiter unread on success.
        let error = core::str::from_utf8(run).expect_err("UTF-8 validation failed");
        if error.error_len().is_some() {
            return Err(input.error(start, ErrorKind::InvalidString));
        }
        let pending = run
            .get(error.valid_up_to()..)
            .expect("partial UTF-8 suffix");
        let width = match pending.first() {
            Some(0xc2..=0xdf) => 2,
            Some(0xe0..=0xef) => 3,
            _ => 4,
        };
        let mut bytes = [0u8; 4];
        let mut filled = pending.len();
        bytes
            .get_mut(..filled)
            .expect("partial code point")
            .copy_from_slice(pending);
        input.advance(n)?;
        while filled < width {
            let Some(b @ 0x80..=0xbf) = input.peek()? else {
                return Err(input.error(start, ErrorKind::InvalidString));
            };
            *bytes.get_mut(filled).expect("code point width") = b;
            filled += 1;
            input.advance(1)?;
        }
        if core::str::from_utf8(bytes.get(..width).expect("code point width")).is_err() {
            return Err(input.error(start, ErrorKind::InvalidString));
        }
    }
}
