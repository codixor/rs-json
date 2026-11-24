use rs_json::{JsonErrorKind, parse};

#[test]
fn keeps_raw_escape_sequences_in_string_view() {
    let parsed = parse(r#"{"text":"line\nvalue","unicode":"\u263A"}"#).unwrap();

    assert_eq!(parsed["text"].as_str(), Some(r"line\nvalue"));
    assert_eq!(parsed["unicode"].as_str(), Some(r"\u263A"));
}

#[test]
fn accepts_escaped_quote_and_backslash() {
    let parsed = parse(r#"{"quote":"say \"hi\"","slash":"a\\b"}"#).unwrap();

    assert_eq!(parsed["quote"].as_str(), Some(r#"say \"hi\""#));
    assert_eq!(parsed["slash"].as_str(), Some(r"a\\b"));
}

#[test]
fn rejects_invalid_unicode_escape() {
    let error = parse(r#"{"bad":"\u12G4"}"#).unwrap_err();
    assert_eq!(error.kind(), JsonErrorKind::InvalidString);
}

#[test]
fn rejects_unescaped_control_character() {
    let error = parse("{\"bad\":\"line\u{0008}break\"}").unwrap_err();
    assert_eq!(error.kind(), JsonErrorKind::InvalidString);
}
