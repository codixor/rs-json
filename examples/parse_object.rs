fn main() {
    let parsed = rs_json::parse(r#"{"name":"json","ready":true}"#).unwrap();

    println!("name = {:?}", parsed["name"].as_str());
    println!("ready = {:?}", parsed["ready"].as_bool());
}
