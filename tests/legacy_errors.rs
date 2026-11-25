//! Compatibility with the published 0.1 parser.
#![cfg(feature = "legacy")]
#![allow(deprecated)]
use rs_json::legacy::{JsonErrorKind, parse};

#[test]
fn rejects_trailing_characters() {
    let error = parse(r#"{"a":1} trailing"#).unwrap_err();

    assert_eq!(error.kind(), JsonErrorKind::TrailingCharacters);
    assert_eq!(error.index(), 8);
}

#[test]
fn rejects_invalid_array_structure() {
    let error = parse("[1,,2]").unwrap_err();
    assert_eq!(error.kind(), JsonErrorKind::InvalidToken);
}

#[test]
fn rejects_invalid_object_structure() {
    let error = parse(r#"{"a" 1}"#).unwrap_err();
    assert_eq!(error.kind(), JsonErrorKind::InvalidObject);
}

#[test]
fn exposes_nearby_extract() {
    let error = parse(r#"{"a":[1,2,]}"#).unwrap_err();
    assert!(error.extract().is_some());
}
