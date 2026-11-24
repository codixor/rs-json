use rs_json::parse;

const SAMPLE: &str = r#"{
  "test": "why not?",
  "obj": {"a": "b", "c": "d"},
  "arr": [1, 2, 3],
  "depth": ["a", ["b", "c"]],
  "emp_a": [],
  "emp_m": {}
}"#;

#[test]
fn traverses_objects_and_arrays() {
    let parsed = parse(SAMPLE).unwrap();

    assert_eq!(parsed["obj"]["a"].as_str(), Some("b"));
    assert_eq!(parsed["arr"][1].as_i128(), Some(2));
    assert_eq!(parsed["depth"][1][1].as_str(), Some("c"));
}

#[test]
fn iterates_entries_and_elements() {
    let parsed = parse(SAMPLE).unwrap();

    let entries: Vec<_> = parsed["obj"]
        .entries()
        .unwrap()
        .map(|(key, value)| (key.as_str(), value.as_str().unwrap()))
        .collect();
    assert_eq!(entries, vec![("a", "b"), ("c", "d")]);

    let values: Vec<_> = parsed["arr"]
        .elements()
        .unwrap()
        .map(|value| value.as_i128().unwrap())
        .collect();
    assert_eq!(values, vec![1, 2, 3]);
}

#[test]
fn missing_items_return_empty_placeholders() {
    let parsed = parse(SAMPLE).unwrap();

    assert!(!parsed["missing"].exists());
    assert_eq!(parsed["missing"][0].as_i128(), None);
}

#[test]
fn empty_containers_still_return_iterators() {
    let parsed = parse(SAMPLE).unwrap();

    assert_eq!(parsed["emp_a"].elements().unwrap().count(), 0);
    assert_eq!(parsed["emp_m"].entries().unwrap().count(), 0);
}
