//! Number conversion. Lexing only validates grammar; conversion happens on
//! access. Integers use a hand-rolled overflow-checked loop (bufjson reports
//! this beats `core` because grammar is pre-validated). Floats forward to
//! `core::str::parse::<f64>` (Eisel–Lemire since Rust 1.55) — do not add
//! `fast-float2`/`lexical-core` unless a canada.json bench shows a ≥15% win.

/// Parse a validated JSON integer literal. `None` if it has a fraction or
/// exponent, or overflows.
pub fn parse_i64(lit: &[u8]) -> Option<i64> {
    let (neg, digits) = match lit.split_first()? {
        (b'-', rest) => (true, rest),
        _ => (false, lit),
    };
    if digits.is_empty() || digits.len() > 19 {
        return None;
    }
    // Agent TODO: 8-digits-at-a-time SWAR conversion for len >= 8, bench vs this loop.
    let mut acc: u64 = 0;
    for &d in digits {
        if !d.is_ascii_digit() {
            return None;
        }
        acc = acc.checked_mul(10)?.checked_add(u64::from(d - b'0'))?;
    }
    if neg {
        if acc == 1u64 << 63 { Some(i64::MIN) } else { i64::try_from(acc).ok().map(|v| -v) }
    } else {
        i64::try_from(acc).ok()
    }
}

/// Parse a validated JSON non-negative integer literal as `u64`.
pub fn parse_u64(lit: &[u8]) -> Option<u64> {
    if lit.is_empty() || lit.len() > 20 {
        return None;
    }
    lit.iter().try_fold(0u64, |acc, &d| {
        d.is_ascii_digit().then_some(())?;
        acc.checked_mul(10)?.checked_add(u64::from(d - b'0'))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ints() {
        assert_eq!(parse_i64(b"0"), Some(0));
        assert_eq!(parse_i64(b"-9223372036854775808"), Some(i64::MIN));
        assert_eq!(parse_i64(b"9223372036854775807"), Some(i64::MAX));
        assert_eq!(parse_i64(b"9223372036854775808"), None);
        assert_eq!(parse_i64(b"1.5"), None);
        assert_eq!(parse_u64(b"18446744073709551615"), Some(u64::MAX));
    }
}
