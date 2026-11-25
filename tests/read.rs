//! Reader and slice parity at refill boundaries.
#![cfg(feature = "read")]

use rs_json::{
    Error, ErrorKind, Limits, Pos,
    config::BufConfig,
    lex::{Lexer, Token, read::ReadLexer, slice::SliceLexer},
    stream::{Event, PullParser},
};
use std::io::{self, Cursor, Read};

const SIZES: [usize; 4] = [1, 7, 64, 8192];

fn config(size: usize) -> BufConfig {
    BufConfig {
        read_buf: size,
        ..BufConfig::default()
    }
}
fn depth(limit: u32) -> Limits {
    Limits {
        max_depth: limit,
        ..Limits::default()
    }
}

#[test]
fn tokens_spans_and_events_match_at_refill_boundaries() {
    let body = r#"{"[key]":[-12.34e+5,"a\\\"b\u0063\ud83d\ude03é日🦀",true,false,null,0]}"#;
    for size in SIZES {
        // Move every part of this short document across a physical refill.
        for tail in 0..body.len() {
            let pad = size - tail % size;
            let src = format!("\n{}{}", " ".repeat(pad - 1), body);
            let bytes = src.as_bytes();
            let mut slice = SliceLexer::new(bytes, false);
            let mut reader = ReadLexer::new(bytes, config(size), depth(2));
            loop {
                let token = slice.next_token().unwrap();
                assert_eq!(reader.next_token(), Ok(token), "size {size}, tail {tail}");
                assert_eq!(reader.span(), slice.span());
                assert_eq!(reader.content(), slice.content());
                if token == Token::Eof {
                    break;
                }
            }
            let mut slice = PullParser::new(SliceLexer::new(bytes, false), depth(2));
            let mut reader = PullParser::from_reader(bytes, config(size), depth(2));
            loop {
                let e = slice.next_event().unwrap();
                assert_eq!(reader.next_event(), Ok(e));
                if matches!(e, Event::Key { .. } | Event::Str { .. } | Event::Num) {
                    assert_eq!(reader.content(), slice.content());
                }
                if e == Event::Eof {
                    break;
                }
            }
        }
    }
}

fn finish<L: Lexer>(mut parser: PullParser<L>) -> Result<(), Error> {
    while parser.next_event()? != Event::Eof {}
    Ok(())
}

#[test]
fn same_depth_for_values_skips_and_multiple_documents() {
    for size in SIZES {
        for limit in [0, 1, 2, 128, 512] {
            let prefix = "\n \t";
            let depth = depth(limit);
            let allowed = format!(
                "{prefix}{}0{}",
                "[".repeat(limit as usize),
                "]".repeat(limit as usize)
            );
            let over = format!(
                "{prefix}{}{{}}{}",
                "[".repeat(limit as usize),
                "]".repeat(limit as usize)
            );
            assert!(
                finish(PullParser::from_reader(
                    allowed.as_bytes(),
                    config(size),
                    depth
                ))
                .is_ok()
            );
            let expected =
                rs_json::Document::parse_with(over.as_bytes(), depth, false).unwrap_err();
            assert_eq!(expected.kind, ErrorKind::DepthLimit);
            assert_eq!(
                expected.pos,
                Pos {
                    offset: 3 + u64::from(limit),
                    line: 2,
                    col: 3 + limit
                }
            );
            assert_eq!(
                finish(PullParser::from_reader(
                    over.as_bytes(),
                    config(size),
                    depth
                )),
                Err(expected)
            );
            let mut skip = PullParser::from_reader(over.as_bytes(), config(size), depth);
            assert_eq!(skip.skip_value(), Err(expected));
            assert_eq!(skip.next_event(), Err(expected));
        }
        let docs = b"{} [0] {\"x\":[]}";
        let mut p = PullParser::from_reader(&docs[..], config(size), depth(1)).multi_document(true);
        for _ in 0..2 {
            p.skip_value().unwrap();
            assert_eq!(p.next_event(), Ok(Event::End));
        }
        assert_eq!(p.skip_value().unwrap_err().kind, ErrorKind::DepthLimit);
    }
}

#[test]
fn malformed_and_truncated_tokens_report_slice_positions() {
    let cases: &[&[u8]] = &[
        b"tru",
        b"faX",
        b"[-00]",
        b"1.",
        b"1e+",
        b"\"unterminated",
        br#""\u12"#,
        br#""\ud800""#,
        br#""\ud800\u00gg""#,
        br#""a\q""#,
        b"\"\xc3\xa9\xe2\x82\"",
        b"\"\xe2\x82",
        b"\"\xf0\x80\x80\x80\"",
        b"{\n  \"key\"\n  true}",
        b"[0,\n\t]",
        b"[] {}",
    ];
    for size in SIZES {
        for &case in cases {
            let mut bytes = vec![b' '; size - 1];
            bytes.extend_from_slice(b"\n");
            bytes.extend_from_slice(case);
            let error = rs_json::parse_bytes(&bytes).unwrap_err();
            assert_eq!(
                finish(PullParser::from_reader(
                    &bytes[..],
                    config(size),
                    Limits::default()
                )),
                Err(error),
                "size {size}, case {case:?}"
            );
        }
    }
}

#[test]
fn raw_token_limit_applies_on_both_sides_of_a_refill() {
    for size in SIZES {
        for len in [0, 1, 7, 64, 8192] {
            let limits = Limits {
                max_token_bytes: len,
                ..Limits::default()
            };
            let full = format!("[\"{}\"]", "x".repeat(len));
            assert!(
                finish(PullParser::from_reader(
                    full.as_bytes(),
                    config(size),
                    limits
                ))
                .is_ok()
            );
            for (text, start) in [
                (format!("[\"{}\"]", "x".repeat(len + 1)), 2),
                (format!("[{}]", "9".repeat(len + 1)), 1),
            ] {
                let mut p = PullParser::from_reader(text.as_bytes(), config(size), limits);
                let err = p.skip_value().unwrap_err();
                assert_eq!(
                    (err.kind, err.offset()),
                    (ErrorKind::SizeLimit, (start + len) as u64)
                );
            }
        }
    }
}

struct ShortReads<'a> {
    src: &'a [u8],
    requested: usize,
    interrupt: bool,
}

impl Read for ShortReads<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        assert_eq!(buf.len(), self.requested);
        self.interrupt = !self.interrupt;
        if self.interrupt {
            return Err(io::ErrorKind::Interrupted.into());
        }
        let n = buf.len().min(3);
        self.src.read(buf.get_mut(..n).unwrap())
    }
}

#[test]
fn retries_interrupted_short_reads_and_preserves_lookahead() {
    for size in SIZES {
        let input = ShortReads {
            src: br#""\u0041" 123,tail"#,
            requested: size,
            interrupt: false,
        };
        let mut lexer = ReadLexer::new(input, config(size), Limits::default());
        assert_eq!(lexer.next_token(), Ok(Token::Str { escaped: true }));
        assert_eq!(lexer.content(), br"\u0041");
        assert_eq!(lexer.next_token(), Ok(Token::Num));
        assert_eq!(lexer.content(), b"123");
        let (reader, mut unread) = lexer.into_inner();
        unread.extend_from_slice(reader.src);
        assert_eq!(unread, b",tail");
    }
}

struct Fails;
impl Read for Fails {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::other("read failed"))
    }
}

#[test]
fn io_error_keeps_stream_position_and_depth_error_needs_no_extra_read() {
    for size in SIZES {
        let start = b"\n \n  \"ab";
        let reader = Cursor::new(start).chain(Fails);
        let mut p = PullParser::from_reader(reader, config(size), Limits::default());
        let err = p.next_event().unwrap_err();
        assert_eq!(
            (err.kind, err.pos),
            (
                ErrorKind::Io,
                Pos {
                    offset: 8,
                    line: 3,
                    col: 6
                }
            )
        );
        assert_eq!(p.next_event(), Err(err));

        let reader = Cursor::new(b"[[").chain(Fails);
        let mut p = PullParser::from_reader(reader, config(size), depth(1));
        assert_eq!(p.next_event(), Ok(Event::ArrayStart));
        assert_eq!(p.next_event().unwrap_err().kind, ErrorKind::DepthLimit);
    }
}
