//! serde codec.
#![allow(
    clippy::indexing_slicing,
    reason = "these assertions exercise public JSON indexing on fixed test fixtures"
)]
use rs_json::{
    JsonSerdeCategory, JsonSerdeError, JsonValue, from_reader, from_slice, from_str, from_value,
    to_string, to_string_pretty, to_value, to_vec, to_vec_pretty,
};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io::{self, Read, Write};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
    roles: Vec<String>,
}

fn sample_user() -> User {
    User {
        id: 7,
        name: String::from("alice"),
        roles: vec![String::from("admin"), String::from("writer")],
    }
}

#[test]
fn serde_roundtrip_from_bytes() {
    let user = sample_user();
    let encoded = to_vec(&user).unwrap();
    let decoded: User = from_slice(&encoded).unwrap();

    assert_eq!(decoded, user);
}

#[test]
fn serde_roundtrip_from_string() {
    let user = sample_user();
    let encoded = to_string(&user).unwrap();
    let decoded: User = from_str(&encoded).unwrap();

    assert_eq!(decoded, user);
}

#[test]
fn serde_value_bridge_roundtrips() {
    let user = sample_user();
    let value = to_value(&user).unwrap();
    let decoded: User = from_value(value).unwrap();

    assert_eq!(decoded, user);
}

#[test]
fn serde_pretty_variants_emit_json() {
    let user = sample_user();
    let pretty_string = to_string_pretty(&user).unwrap();
    let pretty_bytes = to_vec_pretty(&user).unwrap();
    let value: JsonValue = from_str(&pretty_string).unwrap();

    assert!(pretty_string.contains('\n'));
    assert!(pretty_bytes.contains(&b'\n'));
    assert_eq!(value["name"], JsonValue::String(String::from("alice")));
}

#[test]
fn serde_streams_roundtrip_and_preserve_error_categories() {
    let user = sample_user();
    let mut bytes = Vec::new();
    rs_json::to_writer(&mut bytes, &user).unwrap();
    assert_eq!(
        rs_json::from_reader::<User, _>(bytes.as_slice()).unwrap(),
        user
    );
    bytes.clear();
    rs_json::to_writer_pretty(&mut bytes, &user).unwrap();
    assert_eq!(
        rs_json::from_reader::<User, _>(bytes.as_slice()).unwrap(),
        user
    );
    assert!(matches!(
        rs_json::from_reader::<User, _>(&b"["[..]),
        Err(rs_json::JsonSerdeError::Deserialize { .. })
    ));
}

const FIXTURE: &str = "sample-colour";

fn assert_redacted(error: &JsonSerdeError) {
    assert!(!error.to_string().contains(FIXTURE));
    assert!(!format!("{error:?}").contains(FIXTURE));
    assert!(error.source().is_none());
}

#[test]
fn decode_failures_keep_category_and_location() {
    let source = format!("\n\"{FIXTURE}\"");
    let position = Some((2, source.trim_start().len()));
    for error in [
        from_str::<u64>(&source).unwrap_err(),
        from_slice::<u64>(source.as_bytes()).unwrap_err(),
        from_reader::<u64, _>(source.as_bytes()).unwrap_err(),
    ] {
        assert_redacted(&error);
        assert_eq!(error.category(), Some(JsonSerdeCategory::Data));
        assert_eq!(error.position(), position);
    }

    let error = from_value::<u64>(JsonValue::from(FIXTURE)).unwrap_err();
    assert_redacted(&error);
    assert_eq!(error.category(), Some(JsonSerdeCategory::Data));
    assert_eq!(error.position(), None);
    for (json, expected) in [
        ("[}", JsonSerdeCategory::Syntax),
        ("[1", JsonSerdeCategory::Eof),
    ] {
        let error = from_str::<JsonValue>(json).unwrap_err();
        assert_eq!(error.category(), Some(expected));
        assert!(error.position().is_some());
    }
}

struct FailingStream;

impl Read for FailingStream {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::other(FIXTURE))
    }
}

impl Write for FailingStream {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::Error::other(FIXTURE))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn io_diagnostics_omit_custom_error_text() {
    let error = from_reader::<JsonValue, _>(FailingStream).unwrap_err();
    assert_redacted(&error);
    assert_eq!(error.category(), Some(JsonSerdeCategory::Io));

    let error = rs_json::to_writer(FailingStream, &1).unwrap_err();
    assert_redacted(&error);
    assert!(matches!(error, JsonSerdeError::Serialize { .. }));
}

#[test]
fn encoding_diagnostics_omit_custom_serialize_text() {
    struct Rejected;
    impl Serialize for Rejected {
        fn serialize<S: serde::Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom(FIXTURE))
        }
    }
    let error = to_value(&Rejected).unwrap_err();
    assert_redacted(&error);
    assert!(matches!(error, JsonSerdeError::Serialize { .. }));
}
