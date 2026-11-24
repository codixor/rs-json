//! String escape validation and lazy decoding.

use crate::ErrorKind;
use alloc::string::String;

fn hex_quad(bytes: &[u8]) -> Result<u16, ErrorKind> {
    let digits = bytes.get(..4).ok_or(ErrorKind::UnexpectedEof)?;
    digits.iter().try_fold(0u16, |n, &byte| {
        let digit = match byte {
            b'0'..=b'9' => byte - b'0',
            b'a'..=b'f' => byte - b'a' + 10,
            b'A'..=b'F' => byte - b'A' + 10,
            _ => return Err(ErrorKind::InvalidString),
        };
        Ok((n << 4) | u16::from(digit))
    })
}

/// Decode one escape starting immediately after its backslash.
pub(crate) fn escape(bytes: &[u8]) -> Result<(char, usize), ErrorKind> {
    let (&first, tail) = bytes.split_first().ok_or(ErrorKind::UnexpectedEof)?;
    if first == b'u' {
        let high = hex_quad(tail)?;
        let (code, len) = match high {
            0xd800..=0xdbff => {
                let pair = tail
                    .get(4..)
                    .and_then(|s| s.strip_prefix(b"\\u"))
                    .ok_or(ErrorKind::LoneSurrogate)?;
                let low = hex_quad(pair)?;
                if !(0xdc00..=0xdfff).contains(&low) {
                    return Err(ErrorKind::LoneSurrogate);
                }
                (
                    0x10000 + (u32::from(high - 0xd800) << 10) + u32::from(low - 0xdc00),
                    11,
                )
            }
            0xdc00..=0xdfff => return Err(ErrorKind::LoneSurrogate),
            _ => (u32::from(high), 5),
        };
        return char::from_u32(code)
            .map(|ch| (ch, len))
            .ok_or(ErrorKind::InvalidString);
    }
    let ch = match first {
        b'"' => '"',
        b'\\' => '\\',
        b'/' => '/',
        b'b' => '\x08',
        b'f' => '\x0c',
        b'n' => '\n',
        b'r' => '\r',
        b't' => '\t',
        _ => return Err(ErrorKind::InvalidString),
    };
    Ok((ch, 1))
}

// Each item borrows the next plain run and optionally decodes its following escape.
fn pieces(mut raw: &str) -> impl Iterator<Item = (&str, Option<char>)> {
    core::iter::from_fn(move || {
        if raw.is_empty() {
            return None;
        }
        if let Some((plain, tail)) = raw.split_once('\\')
            && let Ok((ch, used)) = escape(tail.as_bytes())
        {
            raw = tail.get(used..).unwrap_or_default();
            return Some((plain, Some(ch)));
        }
        Some((core::mem::take(&mut raw), None))
    })
}

/// Expand escapes in a validated raw string body.
pub fn unescape(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for (plain, escaped) in pieces(raw) {
        out.push_str(plain);
        if let Some(ch) = escaped {
            out.push(ch);
        }
    }
    out
}

/// Compare a validated escaped body with plain text without allocating.
pub fn unescaped_eq(raw: &str, mut other: &str) -> bool {
    for (plain, escaped) in pieces(raw) {
        let Some(rest) = other.strip_prefix(plain) else {
            return false;
        };
        other = rest;
        if let Some(ch) = escaped {
            let Some(rest) = other.strip_prefix(ch) else {
                return false;
            };
            other = rest;
        }
    }
    other.is_empty()
}
