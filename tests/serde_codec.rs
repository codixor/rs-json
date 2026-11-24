use rs_json::{
    JsonValue, from_slice, from_str, from_value, to_string, to_string_pretty, to_value, to_vec,
    to_vec_pretty,
};
use serde::{Deserialize, Serialize};

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
