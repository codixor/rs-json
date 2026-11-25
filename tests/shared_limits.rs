//! Resource boundaries shared by the DOM, event stream and typed decoder.
use rs_format::{Category, FormatError, Limits};
use rs_json::{
    Error, Pos,
    lex::{Lexer, slice::SliceLexer},
    stream::{Event, PullParser},
};

fn drain<L: Lexer>(mut parser: PullParser<L>) -> Result<(), Error> {
    while parser.next_event()? != Event::Eof {}
    Ok(())
}

fn assert_bounds(source: &str, limits: Limits, excess: Option<usize>) {
    fn check<E: FormatError>(actual: Result<(), E>, expected: Option<rs_format::Position>) {
        match (actual, expected) {
            (Ok(()), None) => {}
            (Err(error), Some(position)) => {
                assert_eq!(error.category(), Category::Limit, "{error:?}");
                assert_eq!(error.position(), Some(position));
            }
            (result, position) => panic!("result {result:?} at expected location {position:?}"),
        }
    }

    let expected = excess.map(|offset| Pos::resolve(source.as_bytes(), offset as u64).into());
    let mut reusable = rs_json::Parser::new();
    reusable.parse(source).unwrap();
    reusable.limits = limits;
    check(reusable.parse(source).map(drop), expected);
    check(
        rs_json::parse_with_limits(source, limits).map(drop),
        expected,
    );
    check(
        rs_json::parse_bytes_with_limits(source.as_bytes(), limits).map(drop),
        expected,
    );
    check(
        drain(PullParser::new(
            SliceLexer::new(source.as_bytes(), false),
            limits,
        )),
        expected,
    );
    #[cfg(feature = "read")]
    for size in [1, 2, 3, 7, 64] {
        check(
            drain(PullParser::from_reader(
                source.as_bytes(),
                rs_json::config::BufConfig {
                    read_buf: size,
                    ..Default::default()
                },
                limits,
            )),
            expected,
        );
    }
    #[cfg(feature = "serde")]
    {
        use rs_json::{
            JsonValue, from_reader_with_limits, from_slice_with_limits, from_str_with_limits,
        };
        check(
            from_str_with_limits::<JsonValue>(source, limits).map(drop),
            expected,
        );
        check(
            from_slice_with_limits::<JsonValue>(source.as_bytes(), limits).map(drop),
            expected,
        );
        check(
            from_reader_with_limits::<JsonValue, _>(source.as_bytes(), limits).map(drop),
            expected,
        );
    }
}

#[test]
fn keys_strings_numbers_and_keywords_obey_the_encoded_token_budget() {
    for (source, start, width) in [
        (r#""abc""#, 1, 3),
        (r#"{"abcd":0}"#, 2, 4),
        (r#""a\"b""#, 1, 4),
        (r#""\u0061""#, 1, 6),
        (r#""\ud800\udc00""#, 1, 12),
        ("\"a\u{1f980}\"", 1, 5),
        ("-0.1e+3", 0, 7),
        ("false", 0, 5),
    ] {
        for cap in 0..=width + 1 {
            assert_bounds(
                source,
                Limits {
                    max_token_bytes: cap,
                    ..Limits::default()
                },
                (cap < width).then_some(start + cap),
            );
        }
    }
    assert_bounds(
        r#"["",{}]"#,
        Limits {
            max_token_bytes: 0,
            max_expansion: 0,
            ..Limits::default()
        },
        None,
    );
}

#[test]
fn document_budget_includes_every_byte_and_whitespace() {
    let source = " \n{\"id\":\"\\u0061\"}\n ";
    for cap in 0..=source.len() + 1 {
        assert_bounds(
            source,
            Limits {
                max_document_bytes: cap as u64,
                ..Limits::default()
            },
            (cap < source.len()).then_some(cap),
        );
    }
}

#[test]
fn the_first_exceeded_limit_wins_across_inputs() {
    for (source, depth, document, token, first) in [
        ("[[0]]", 1, 4, 8, 1),
        (r#"["abcd"]"#, 1, 6, 2, 4),
        ("[true]", 1, 4, 2, 3),
        ("[true]", 1, 2, 3, 2),
    ] {
        assert_bounds(
            source,
            Limits {
                max_depth: depth,
                max_document_bytes: document,
                max_token_bytes: token,
                ..Default::default()
            },
            Some(first),
        );
    }
}

#[test]
fn incomplete_literal_and_number_prefixes_report_the_input_end() {
    for prefix in ["t", "tru", "fa", "nul", "-", "1.", "1e", "1E-"] {
        let input = format!("[\n {prefix}");
        let wanted = Pos::resolve(input.as_bytes(), input.len() as u64);
        let check = |error: Error| {
            assert_eq!(error.category(), Category::Eof, "{input:?}");
            assert_eq!(error.pos, wanted, "{input:?}");
        };
        check(rs_json::parse(&input).unwrap_err());
        check(rs_json::parse_bytes(input.as_bytes()).unwrap_err());
        check(rs_json::Parser::new().parse(&input).unwrap_err());
        #[cfg(feature = "read")]
        for read_buf in [1, 3, 64] {
            check(
                drain(PullParser::from_reader(
                    input.as_bytes(),
                    rs_json::config::BufConfig {
                        read_buf,
                        ..Default::default()
                    },
                    Limits::default(),
                ))
                .unwrap_err(),
            );
        }
    }
    for invalid in ["trX", "fa]", "1.]", "1e+ "] {
        assert_eq!(
            rs_json::parse(invalid).unwrap_err().category(),
            Category::Syntax
        );
    }
}

#[test]
fn multi_document_accepts_json_whitespace_as_framing() {
    let parse = |input: &[u8]| {
        drain(
            PullParser::new(SliceLexer::new(input, false), Limits::default()).multi_document(true),
        )
    };
    assert!(parse(b"{}[]\n 42\r\ntrue").is_ok());
    let error = parse(b"\x1e{}\n").unwrap_err();
    assert_eq!((error.category(), error.offset()), (Category::Syntax, 0));
}

#[test]
fn depth_is_inclusive_and_json_charges_zero_expansion() {
    let source = "\n [\"[{}]\",{\"x\":\"\\u0061\"}]";
    for (depth, excess) in [(0, Some(2)), (1, Some(10)), (2, None)] {
        assert_bounds(
            source,
            Limits {
                max_depth: depth,
                max_expansion: 0,
                ..Limits::default()
            },
            excess,
        );
    }
}

#[test]
fn multi_document_resets_byte_budget_with_absolute_offsets() {
    fn check<L: Lexer>(parser: PullParser<L>) {
        let mut parser = parser.multi_document(true);
        for _ in 0..2 {
            parser.skip_value().unwrap();
            assert_eq!(parser.next_event().unwrap(), Event::End);
        }
        let error = parser.skip_value().unwrap_err();
        assert_eq!(error.kind, rs_json::ErrorKind::SizeLimit);
        assert_eq!(
            error.pos,
            Pos {
                offset: 15,
                line: 2,
                col: 5
            }
        );
    }
    let src = b"\"abc\" 1234\n  2345";
    let limits = Limits {
        max_document_bytes: 5,
        ..Limits::default()
    };
    check(PullParser::new(SliceLexer::new(src, false), limits));
    #[cfg(feature = "read")]
    for size in [1, 3, 64] {
        check(PullParser::from_reader(
            src.as_slice(),
            rs_json::config::BufConfig {
                read_buf: size,
                ..Default::default()
            },
            limits,
        ));
    }
}

#[test]
fn native_diagnostics_keep_coordinates_and_discard_context() {
    let source = "{\n\"private-marker\":]}";
    let error = rs_json::parse(source).unwrap_err();
    let diagnostic: &dyn FormatError = &error;
    assert_eq!(diagnostic.category(), Category::Syntax);
    assert_eq!(
        diagnostic.position(),
        Some(Pos::resolve(source.as_bytes(), 19).into())
    );
    assert!(!format!("{diagnostic} {diagnostic:?}").contains("private-marker"));
    assert!(diagnostic.source().is_none());
    assert_eq!(rs_json::parse("[").unwrap_err().category(), Category::Eof);
}

#[cfg(feature = "serde")]
#[test]
fn typed_recursion_ceiling_accepts_the_shared_default_inclusively() {
    use rs_json::{JsonValue, from_reader_with_limits, from_slice_with_limits};
    for requested in [127, 128, 1024] {
        let effective = requested.min(128);
        let limits = Limits {
            max_depth: requested,
            ..Limits::default()
        };
        for depth in [effective, effective + 1] {
            let width = usize::try_from(depth).unwrap();
            let text = format!("{}0{}", "[".repeat(width), "]".repeat(width));
            for result in [
                from_slice_with_limits::<JsonValue>(text.as_bytes(), limits),
                from_reader_with_limits::<JsonValue, _>(text.as_bytes(), limits),
            ] {
                if depth == effective {
                    assert!(result.is_ok());
                } else {
                    let error = result.unwrap_err();
                    assert_eq!(FormatError::category(&error), Category::Limit);
                    assert_eq!(
                        FormatError::position(&error).unwrap().offset,
                        Some(u64::from(effective))
                    );
                }
            }
        }
    }
}
