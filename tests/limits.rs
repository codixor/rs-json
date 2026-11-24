//! Safety regressions found in the rust-misc rs-json review.

#[test]
fn configured_depth_limit_is_reported() {
    let limits = rs_json::Limits {
        max_depth: 2,
        ..Default::default()
    };
    let err = rs_json::Document::parse_with(b"[[[]]]", limits, true).unwrap_err();
    assert_eq!(err.kind, rs_json::ErrorKind::DepthLimit);
}

#[test]
fn escaped_key_lookup() {
    // The source spells the first letter with a Unicode escape.
    let doc = rs_json::parse(r#"{"\u0061b":1}"#).unwrap();
    assert_eq!(doc.root().get("ab").and_then(|v| v.as_i64()), Some(1));
}

#[test]
fn lone_surrogate_rejected() {
    // Old parser accepted "\ud800"; serde_json rejects it.
    let err = rs_json::parse(r#""\ud800""#).unwrap_err();
    assert_eq!(err.kind, rs_json::ErrorKind::LoneSurrogate);
}

#[test]
fn node_is_16_bytes() {
    assert_eq!(std::mem::size_of::<rs_json::dom::Node>(), 16);
}

fn limits(depth: u32) -> rs_json::Limits {
    rs_json::Limits {
        max_depth: depth,
        ..Default::default()
    }
}

#[test]
fn zero_allows_only_root_scalars() {
    for src in ["null", "true", "0", r#""[{}]""#] {
        assert!(rs_json::Document::parse_with(src.as_bytes(), limits(0), true).is_ok());
    }
    for src in [b" []".as_slice(), b" {}"] {
        let err = rs_json::Document::parse_with(src, limits(0), false).unwrap_err();
        assert_eq!(
            (err.kind, err.offset()),
            (rs_json::ErrorKind::DepthLimit, 1)
        );
    }
}

#[test]
fn mixed_siblings_release_their_depth_and_keep_their_tape_ranges() {
    let text = r#"{"items":[{"a":"\"[{}]"},{"b":[]},null],"tail":[1,2]}"#;
    let doc = rs_json::Document::parse_with(text.as_bytes(), limits(4), false).unwrap();
    let root = doc.root();
    assert_eq!(root.raw(), text);
    let items = root.get("items").unwrap();
    assert_eq!(
        items.at(0).unwrap().get("a").unwrap().as_str().as_deref(),
        Some("\"[{}]")
    );
    assert_eq!(items.at(1).unwrap().raw(), r#"{"b":[]}"#);
    assert!(items.at(2).unwrap().is_null());
    assert_eq!(root.get("tail").unwrap().at(1).unwrap().as_i64(), Some(2));
    let err = rs_json::Document::parse_with(text.as_bytes(), limits(3), false).unwrap_err();
    assert_eq!(err.kind, rs_json::ErrorKind::DepthLimit);
    assert_eq!(err.offset(), text.find("[]").unwrap() as u64);
}

#[test]
fn owned_bytes_and_reusable_parser_agree_at_default_boundary() {
    let depth = rs_json::Limits::default().max_depth as usize;
    let arrays = |levels| format!("{}null{}", "[".repeat(levels), "]".repeat(levels));
    let at = arrays(depth);
    let over = arrays(depth + 1);
    let mut reusable = rs_json::Parser::new();
    let expected = rs_json::parse(&over).unwrap_err();
    assert_eq!(expected.kind, rs_json::ErrorKind::DepthLimit);
    assert_eq!(expected.offset(), depth as u64);
    assert_eq!(rs_json::parse_bytes(over.as_bytes()).unwrap_err(), expected);
    assert_eq!(reusable.parse(&over).unwrap_err(), expected);
    assert_eq!(reusable.parse(&at).unwrap().tape.len(), depth + 1);
    assert_eq!(rs_json::parse(&at).unwrap().node_count(), depth + 1);
    assert_eq!(
        rs_json::parse_bytes(at.as_bytes()).unwrap().node_count(),
        depth + 1
    );
}

#[test]
fn error_location_and_reuse_after_lowering_the_limit() {
    let mut parser = rs_json::Parser::new();
    parser.limits = limits(1);
    let e = parser.parse("{\n  \"x\": []}").unwrap_err();
    assert_eq!(e.kind, rs_json::ErrorKind::DepthLimit);
    assert_eq!((e.pos.offset, e.pos.line, e.pos.col), (9, 2, 8));
    assert!(parser.parse("[1]").is_ok());
    parser.limits = limits(0);
    assert!(parser.parse("false").is_ok());
    assert_eq!(
        parser.parse("{}").unwrap_err().kind,
        rs_json::ErrorKind::DepthLimit
    );
}

#[test]
fn skipping_and_multiple_roots_obey_the_same_limit() {
    use rs_json::{
        lex::slice::SliceLexer,
        stream::{Event, PullParser},
    };
    let mut p = PullParser::new(SliceLexer::new(br#"{"x":[{}]}"#, false), limits(2));
    assert_eq!(p.next_event().unwrap(), Event::ObjectStart);
    assert_eq!(p.next_event().unwrap(), Event::Key { escaped: false });
    let err = p.skip_value().unwrap_err();
    assert_eq!(
        (err.kind, err.offset()),
        (rs_json::ErrorKind::DepthLimit, 6)
    );
    assert_eq!(p.next_event().unwrap_err(), err);

    let mut p =
        PullParser::new(SliceLexer::new(b"[{}] {} [[[]]]", false), limits(2)).multi_document(true);
    for _ in 0..2 {
        p.skip_value().unwrap();
        assert_eq!(p.next_event().unwrap(), Event::End);
    }
    assert_eq!(
        p.skip_value().unwrap_err().kind,
        rs_json::ErrorKind::DepthLimit
    );
}

#[test]
fn syntax_errors_and_invalid_bytes_are_distinct_from_depth_errors() {
    for src in [
        "",
        "[}",
        "{]",
        "[1,]",
        r#"{"x":1,}"#,
        r#"{"x" 2}"#,
        "[1 2]",
        "{}[]",
        "01",
        "1.",
        "1e+",
        "trux",
        "[  \t\u{0008}      0]",
        r#""\q""#,
    ] {
        let e = rs_json::parse(src).unwrap_err();
        assert_ne!(e.kind, rs_json::ErrorKind::DepthLimit, "{src:?}");
    }
    for hint in [false, true] {
        let e = rs_json::Document::parse_with(b"{\"\xff\":0}", limits(1), hint).unwrap_err();
        assert_eq!(e.kind, rs_json::ErrorKind::InvalidString);
    }
}

proptest::proptest! {
    #[test]
    fn generated_containers_share_an_exact_depth_across_entry_points(
        objects in proptest::collection::vec(proptest::bool::ANY, 0..12),
        leaf in ".{0,24}",
    ) {
        use rs_json::{lex::slice::SliceLexer, stream::{Event, PullParser}};
        use proptest::prelude::*;
        let mut value = serde_json::Value::String(leaf.clone());
        for object in &objects {
            value = if *object { serde_json::json!({"key": value}) } else { serde_json::json!([value]) };
        }
        let text = value.to_string();
        let depth = u32::try_from(objects.len()).unwrap();
        let mut parser = rs_json::Parser::new();
        parser.limits = limits(depth);
        prop_assert!(parser.parse(&text).is_ok());
        let doc = rs_json::Document::parse_with(text.as_bytes(), limits(depth), false).unwrap();
        let mut cursor = doc.root();
        for &object in objects.iter().rev() {
            cursor = if object { cursor.get("key") } else { cursor.at(0) }.unwrap();
        }
        prop_assert_eq!(cursor.as_str().unwrap(), leaf.as_str());
        let mut pull = PullParser::new(SliceLexer::new(text.as_bytes(), false), limits(depth));
        pull.skip_value().unwrap();
        prop_assert_eq!(pull.next_event().unwrap(), Event::End);
        prop_assert_eq!(pull.next_event().unwrap(), Event::Eof);
        if depth > 0 {
            parser.limits = limits(depth - 1);
            let error = parser.parse(&text).unwrap_err();
            prop_assert_eq!(error.kind, rs_json::ErrorKind::DepthLimit);
            let mut pull = PullParser::new(SliceLexer::new(text.as_bytes(), false), parser.limits);
            prop_assert_eq!(pull.skip_value().unwrap_err(), error);
        }
    }
}

#[test]
fn strings_and_integer_hints_survive_nested_values() {
    use rs_json::dom::{Node, NodeTag};
    let text = r#"{"\u0061":["\ud83d\ude03",-1,2.0,3e4]}"#;
    let doc = rs_json::parse(text).unwrap();
    let array = doc.root().get("a").unwrap();
    assert_eq!(array.at(0).unwrap().as_str().as_deref(), Some("\u{1f603}"));
    let mut parser = rs_json::Parser::new();
    let flags: Vec<_> = parser
        .parse(text)
        .unwrap()
        .tape
        .iter()
        .filter(|node| node.tag == NodeTag::Num)
        .map(|node| node.flags & Node::NUM_INT != 0)
        .collect();
    assert_eq!(flags, [true, false, false]);
}

#[test]
fn custom_depth_above_default_counts_mixed_containers() {
    let pairs = 260;
    let text = format!("{}0{}", r#"{"v":["#.repeat(pairs), "]}".repeat(pairs));
    let depth = u32::try_from(2 * pairs).unwrap();
    let permitted = rs_json::Document::parse_with(text.as_bytes(), limits(depth), false).unwrap();
    assert_eq!(permitted.node_count(), 3 * pairs + 1);
    let denied =
        rs_json::Document::parse_with(text.as_bytes(), limits(depth - 1), false).unwrap_err();
    assert_eq!(denied.kind, rs_json::ErrorKind::DepthLimit);
    assert_eq!(denied.offset(), text.rfind('[').unwrap() as u64);
}
