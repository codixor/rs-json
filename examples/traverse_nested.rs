//! Traverse nested.
fn main() {
    let parsed = rs_json::parse(r#"{"outer":{"inner":["a","b","c"]}}"#).unwrap();

    for value in parsed
        .root()
        .get("outer")
        .unwrap()
        .get("inner")
        .unwrap()
        .elements()
        .unwrap()
    {
        println!("{:?}", value.as_str());
    }
}
