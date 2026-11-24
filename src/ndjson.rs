//! Newline-delimited JSON (feature `ndjson`). Lines are split with
//! `memchr::memchr_iter(b'\n', ..)` (newlines cannot appear raw inside JSON
//! strings, so splitting before parsing is safe) and each line is parsed with
//! a shared [`crate::Parser`] to reuse the tape. Optional rayon-free chunked
//! parallelism can be added later behind its own feature.

/// Iterate non-empty lines of an NDJSON buffer (`\r\n` tolerated).
pub fn lines(buf: &[u8]) -> impl Iterator<Item = &[u8]> {
    let mut start = 0;
    memchr::memchr_iter(b'\n', buf)
        .chain(core::iter::once(buf.len()))
        .filter_map(move |end| {
            let line = buf.get(start..end)?;
            start = end + 1;
            let line = line.strip_suffix(b"\r").unwrap_or(line);
            (!line.iter().all(u8::is_ascii_whitespace)).then_some(line)
        })
}
