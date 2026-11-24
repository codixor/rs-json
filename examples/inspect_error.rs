fn main() {
    match rs_json::parse(r#"{"bad":[1,2,]}"#) {
        Ok(_) => println!("parsed"),
        Err(error) => {
            println!("kind: {:?}", error.kind());
            println!("index: {}", error.index());
            println!("extract: {:?}", error.extract());
        }
    }
}
