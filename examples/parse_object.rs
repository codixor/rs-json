//! Parse object.
fn main() {
    let parsed = rs_json::parse(r#"{"name":"json","ready":true}"#).unwrap();

    println!(
        "name = {:?}",
        parsed.root().get("name").and_then(|v| v.as_str())
    );
    println!(
        "ready = {:?}",
        parsed.root().get("ready").and_then(|v| v.as_bool())
    );
}
