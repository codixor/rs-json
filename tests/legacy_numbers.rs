//! Compatibility with the published 0.1 parser.
#![allow(
    clippy::indexing_slicing,
    reason = "these assertions exercise public JSON indexing on fixed test fixtures"
)]
#![cfg(feature = "legacy")]
#![allow(deprecated)]
use rs_json::legacy::{JsonErrorKind, parse};

#[test]
fn parses_integer_float_and_exponent_numbers() {
    let parsed = parse(r#"{"int":-234,"float":4.2344,"exp":3.3e-21,"exp2":3.7391238E24}"#).unwrap();

    assert_eq!(parsed["int"].as_i128(), Some(-234));
    assert_eq!(parsed["float"].as_f64(), Some(4.2344));
    assert!((parsed["exp"].as_f64().unwrap() / 3.3e-21 - 1.0).abs() < 1e-8);
    assert!((parsed["exp2"].as_f64().unwrap() / 3.739_123_8e24 - 1.0).abs() < 1e-8);
}

#[test]
fn rejects_leading_zero_numbers() {
    let error = parse(r#"{"bad":01}"#).unwrap_err();
    assert_eq!(error.kind(), JsonErrorKind::InvalidNumber);
}

#[test]
fn rejects_invalid_exponent() {
    let error = parse("1e").unwrap_err();
    assert_eq!(error.kind(), JsonErrorKind::InvalidNumber);
}
