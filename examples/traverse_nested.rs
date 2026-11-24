fn main() {
    let parsed = rs_json::parse(r#"{"outer":{"inner":["a","b","c"]}}"#).unwrap();

    for value in parsed["outer"]["inner"].elements().unwrap() {
        println!("{:?}", value.as_str());
    }
}
