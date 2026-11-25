//! Newline-delimited JSON (feature `ndjson`). Lines are split with
//! `memchr::memchr_iter(b'\n', ..)` (newlines cannot appear raw inside JSON
//! strings, so splitting before parsing is safe) and each line is parsed with
//! a shared [`crate::Parser`] to reuse the tape. Optional rayon-free chunked
//! parallelism can be added later behind its own feature.

/// Iterate non-empty lines of an NDJSON buffer (`\r\n` tolerated).
pub fn lines(buf: &[u8]) -> impl Iterator<Item = &[u8]> {
    records(buf).map(|(_, line)| line)
}

fn records(buf: &[u8]) -> impl Iterator<Item = (usize, &[u8])> {
    let mut start = 0;
    memchr::memchr_iter(b'\n', buf)
        .chain(core::iter::once(buf.len()))
        .filter_map(move |end| {
            let offset = start;
            let line = buf.get(start..end)?;
            start = end.saturating_add(1);
            let line = line.strip_suffix(b"\r").unwrap_or(line);
            (!line.iter().all(|b| matches!(b, b' ' | b'\t' | b'\r'))).then_some((offset, line))
        })
}

/// Parse each nonblank record with the same parser and borrow its tape for the callback.
///
/// # Errors
/// Returns the first invalid record's error with a position relative to the entire buffer.
pub fn for_each_document<'a>(
    parser: &mut crate::Parser,
    buf: &'a [u8],
    mut visit: impl FnMut(crate::dom::DocRef<'_, 'a>),
) -> Result<(), crate::Error> {
    for (offset, line) in records(buf) {
        let text = core::str::from_utf8(line).map_err(|err| {
            crate::Error::at(
                buf,
                offset + err.valid_up_to(),
                crate::ErrorKind::InvalidString,
            )
        })?;
        let doc = parser.parse(text).map_err(|err| crate::Error {
            kind: err.kind,
            pos: crate::Pos::resolve(buf, offset as u64 + err.offset()),
        })?;
        visit(doc);
    }
    Ok(())
}
