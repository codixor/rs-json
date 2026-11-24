//! Safety regressions found in the rust-misc rs-json review.

#[test]
#[ignore = "enable when dom::build lands (G1)"]
fn configured_depth_limit_is_reported() {
    let limits = rs_json::Limits { max_depth: 2, ..Default::default() };
    let err = rs_json::Document::parse_with(b"[[[]]]", limits, true).unwrap_err();
    assert_eq!(err.kind, rs_json::ErrorKind::DepthLimit);
}

#[test]
#[ignore = "enable when dom::build lands (G1)"]
fn escaped_key_lookup() {
    // Old parser: doc["ab"] did not find the key written as "ab".
    let doc = rs_json::parse(r#"{"ab":1}"#).unwrap();
    assert_eq!(doc.root().get("ab").and_then(|v| v.as_i64()), Some(1));
}

#[test]
#[ignore = "enable when lexer lands (G1)"]
fn lone_surrogate_rejected() {
    // Old parser accepted "\ud800"; serde_json rejects it.
    let err = rs_json::parse(r#""\ud800""#).unwrap_err();
    assert_eq!(err.kind, rs_json::ErrorKind::LoneSurrogate);
}

#[test]
fn node_is_16_bytes() {
    assert_eq!(std::mem::size_of::<rs_json::dom::Node>(), 16);
}
