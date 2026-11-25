//! Compatibility with the published 0.1 parser.
#![allow(
    clippy::indexing_slicing,
    reason = "these assertions exercise public JSON indexing on fixed test fixtures"
)]
#![cfg(feature = "legacy")]
#![allow(deprecated)]
use std::fs;
use std::path::PathBuf;

use rs_json::legacy::{JsonErrorKind, parse};

fn fixture(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
        .join(name);
    fs::read_to_string(path).unwrap()
}

#[test]
fn parses_valid_golden_fixture() {
    let source = fixture("valid_nested.json");
    let parsed = parse(&source).unwrap();

    assert_eq!(parsed["meta"]["name"].as_str(), Some("json"));
    assert_eq!(parsed["meta"]["versions"][2].as_i128(), Some(3));
    assert_eq!(parsed["flags"][1].as_bool(), Some(false));
}

#[test]
fn rejects_invalid_golden_fixture() {
    let source = fixture("invalid_trailing.json");
    let error = parse(&source).unwrap_err();

    assert_eq!(error.kind(), JsonErrorKind::TrailingCharacters);
}

#[test]
fn parses_escape_heavy_golden_fixture() {
    let source = fixture("escaped_strings.json");
    let parsed = parse(&source).unwrap();

    assert_eq!(parsed["path"].as_str(), Some(r"C:\\temp\\json"));
    assert_eq!(parsed["quote"].as_str(), Some(r#"say \"hi\""#));
}
