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
fn finite_floats_roundtrip_bit_for_bit_across_text_and_dom() {
    for original in [
        51.248_178_375_505_404_f64,
        -93.311_370_376_880_33,
        f64::MIN_POSITIVE,
    ] {
        let encoded = to_string(&original).unwrap();
        let document = rs_json::parse(&encoded).unwrap();
        for decoded in [
            from_str::<f64>(&encoded).unwrap(),
            from_slice::<f64>(encoded.as_bytes()).unwrap(),
            from_reader::<f64, _>(encoded.as_bytes()).unwrap(),
            document.root().as_f64().unwrap(),
            document.root().to_serde().unwrap().as_f64().unwrap(),
        ] {
            assert_eq!(decoded.to_bits(), original.to_bits(), "{encoded}");
        }
    }
}

#[test]
fn numeric_range_is_a_data_error_in_original_source_coordinates() {
    use rs_format::{Category, FormatError};
    let input = "{\n  \"é\": [1e999]\n}";
    let doc = rs_json::parse(input).unwrap();
    let direct = from_str::<JsonValue>(input).unwrap_err();
    let byte_column = input.lines().nth(1).unwrap().find(']').unwrap();
    for error in [
        direct,
        from_slice::<JsonValue>(input.as_bytes()).unwrap_err(),
        doc.root().to_serde().unwrap_err(),
        doc.root()
            .get("é")
            .unwrap()
            .at(0)
            .unwrap()
            .to_serde()
            .unwrap_err(),
    ] {
        assert_eq!(FormatError::category(&error), Category::Data);
        assert_eq!(error.position(), Some((2, byte_column)));
    }
    let reader_error = from_reader::<JsonValue, _>(input.as_bytes()).unwrap_err();
    let backend = serde_json::from_reader::<_, JsonValue>(input.as_bytes()).unwrap_err();
    assert_eq!(FormatError::category(&reader_error), Category::Data);
    assert_eq!(
        reader_error.position(),
        Some((backend.line(), backend.column()))
    );
    let too_large = "340282366920938463463374607431768211456";
    assert_eq!(
        FormatError::category(&from_str::<u128>(too_large).unwrap_err()),
        Category::Data
    );
    assert_eq!(
        FormatError::category(&from_str::<f64>("1.e2").unwrap_err()),
        Category::Syntax
    );
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

#[test]
fn shared_diagnostics_expose_only_known_coordinates_and_categories() {
    use rs_format::{Category, FormatError};
    let source = format!("\n\"{FIXTURE}\"");
    let rejected = from_str::<u64>(&source).unwrap_err();
    let error: &dyn FormatError = &rejected;
    assert_eq!(error.category(), Category::Data);
    let position = error.position().unwrap();
    assert_eq!(position.offset, None);
    assert_eq!(position.line.unwrap().get(), 2);
    assert_eq!(position.column.unwrap().get() as usize, source.len() - 1);

    let limit = rs_json::from_reader_with_limits::<String, _>(
        source.as_bytes(),
        rs_json::Limits {
            max_token_bytes: 2,
            ..Default::default()
        },
    )
    .unwrap_err();
    assert_redacted(&limit);
    assert_eq!(FormatError::category(&limit), Category::Limit);
    assert_eq!(FormatError::position(&limit).unwrap().offset, Some(4));
    assert_eq!(
        FormatError::category(&rs_json::to_writer(FailingStream, &1).unwrap_err()),
        Category::Io
    );
    assert_eq!(
        FormatError::category(&from_reader::<u64, _>(FailingStream).unwrap_err()),
        Category::Io
    );
    assert_eq!(
        FormatError::category(&from_str::<JsonValue>("[}").unwrap_err()),
        Category::Syntax
    );
    assert_eq!(
        FormatError::category(&from_str::<JsonValue>("[").unwrap_err()),
        Category::Eof
    );

    let borrowed: &str =
        rs_json::from_str_with_limits(r#""borrowed""#, rs_json::Limits::default()).unwrap();
    assert_eq!(borrowed, "borrowed");
}
