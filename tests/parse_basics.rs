use rs_json::{JsonType, parse};

#[test]
fn parses_object_root() {
    let parsed = parse(r#"{"name":"json","ready":true,"count":3}"#).unwrap();

    assert_eq!(parsed.kind(), JsonType::JsonMap);
    assert_eq!(parsed["name"].as_str(), Some("json"));
    assert_eq!(parsed["ready"].as_bool(), Some(true));
    assert_eq!(parsed["count"].as_i128(), Some(3));
}

#[test]
fn parses_top_level_scalars() {
    assert_eq!(parse(r#""hello""#).unwrap().as_str(), Some("hello"));
    assert_eq!(parse("false").unwrap().as_bool(), Some(false));
    assert_eq!(parse("42").unwrap().as_i128(), Some(42));
}

#[test]
fn rejects_empty_input() {
    let error = parse("   ").unwrap_err();
    assert_eq!(error.kind(), rs_json::JsonErrorKind::UnexpectedEof);
}
