//! String escape validation and lazy decoding.

use crate::ErrorKind;
use alloc::string::String;

pub(crate) enum EscapeError {
    Invalid(ErrorKind),
    Incomplete(ErrorKind),
}

use EscapeError::{Incomplete, Invalid};

// Invalid digits have the high bit set, so all four lanes can be checked together.
#[expect(
    clippy::indexing_slicing,
    reason = "only ASCII hex digits index the 256-entry table"
)]
const HEX: [u8; 256] = {
    let mut table = [0xff; 256];
    let mut digit = 0u8;
    while digit < 16 {
        if digit < 10 {
            table[(b'0' + digit) as usize] = digit;
        } else {
            table[(b'a' + digit - 10) as usize] = digit;
            table[(b'A' + digit - 10) as usize] = digit;
        }
        digit += 1;
    }
    table
};

#[inline]
fn hex_quad(bytes: &[u8]) -> Result<u16, EscapeError> {
    let digits = bytes
        .first_chunk::<4>()
        .ok_or(Incomplete(ErrorKind::UnexpectedEof))?;
    let [a, b, c, d] = digits.map(|byte| {
        *HEX.get(usize::from(byte))
            .expect("256 entries cover every byte")
    });
    if (a | b | c | d) == 0xff {
        return Err(Invalid(ErrorKind::InvalidString));
    }
    Ok(u16::from_be_bytes([(a << 4) | b, (c << 4) | d]))
}

#[inline]
pub(crate) fn validated_unicode_len(bytes: &[u8]) -> Option<usize> {
    let unit = hex_quad(bytes).ok()?;
    match unit {
        0xd800..=0xdbff => {
            let low = bytes.get(4..)?.strip_prefix(b"\\u")?;
            (hex_quad(low).ok()? & 0xfc00 == 0xdc00).then_some(10)
        }
        0xdc00..=0xdfff => None,
        _ => Some(4),
    }
}

/// Decode one escape starting immediately after its backslash.
#[inline]
pub(crate) fn escape(bytes: &[u8]) -> Result<(char, usize), ErrorKind> {
    escape_prefix(bytes).map_err(|e| match e {
        Invalid(kind) | Incomplete(kind) => kind,
    })
}

// A refill can complete a partial escape. At true EOF, report its original error kind.
#[inline]
pub(crate) fn escape_prefix(bytes: &[u8]) -> Result<(char, usize), EscapeError> {
    let (&first, tail) = bytes
        .split_first()
        .ok_or(Incomplete(ErrorKind::UnexpectedEof))?;
    if first == b'u' {
        let high = hex_quad(tail)?;
        let (code, len) = match high {
            0xd800..=0xdbff => {
                let rest = tail.get(4..).unwrap_or_default();
                if rest.len() < 2 && b"\\u".starts_with(rest) {
                    return Err(Incomplete(ErrorKind::LoneSurrogate));
                }
                let pair = rest
                    .strip_prefix(b"\\u")
                    .ok_or(Invalid(ErrorKind::LoneSurrogate))?;
                let low = hex_quad(pair)?;
                if !(0xdc00..=0xdfff).contains(&low) {
                    return Err(Invalid(ErrorKind::LoneSurrogate));
                }
                (
                    0x10000 + (u32::from(high - 0xd800) << 10) + u32::from(low - 0xdc00),
                    11,
                )
            }
            0xdc00..=0xdfff => return Err(Invalid(ErrorKind::LoneSurrogate)),
            _ => (u32::from(high), 5),
        };
        return char::from_u32(code)
            .map(|ch| (ch, len))
            .ok_or(Invalid(ErrorKind::InvalidString));
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
        _ => return Err(Invalid(ErrorKind::InvalidString)),
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
