//! Cross-check the pinned `JSONTestSuite` against the DOM and both pull sources.
#![cfg(feature = "read")]

use rs_json::{
    Limits, Parser,
    config::BufConfig,
    lex::slice::SliceLexer,
    stream::{Event, PullParser},
};
use std::{collections::BTreeMap, fs, path::PathBuf};

#[test]
#[ignore = "requires the pinned JSONTestSuite test_parsing directory; run explicitly in CI"]
fn json_test_suite() {
    let dir = PathBuf::from(
        std::env::var_os("RS_JSON_CONFORMANCE_DIR")
            .expect("set RS_JSON_CONFORMANCE_DIR to JSONTestSuite/test_parsing"),
    );
    let mut choices: BTreeMap<_, _> = include_str!("conformance/decisions.toml")
        .lines()
        .filter_map(|line| line.split_once(" = "))
        .map(|(key, value)| (key.trim_matches('"'), value.parse::<bool>().unwrap()))
        .collect();
    let mut seen = 0;
    let mut parser = Parser::new();
    for path in fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
    {
        if path.extension().is_none_or(|ext| ext != "json") {
            continue;
        }
        let name = path.file_name().unwrap().to_str().unwrap();
        let expected = match name.as_bytes().first() {
            Some(b'y') => true,
            Some(b'n') => false,
            Some(b'i') => choices
                .remove(name)
                .expect("missing implementation-defined decision"),
            _ => panic!("unexpected test case {name}"),
        };
        let bytes = fs::read(&path).unwrap();
        let result = rs_json::parse_bytes(&bytes).map(|_| ());
        assert_eq!(result.is_ok(), expected, "{name}: {result:?}");
        if let Ok(text) = std::str::from_utf8(&bytes) {
            assert_eq!(rs_json::parse(text).map(|_| ()), result, "{name}");
            assert_eq!(parser.parse(text).map(|_| ()), result, "{name}");
        }
        for size in [1, 7, 64, 8192] {
            let limits = Limits::default();
            let mut reference = PullParser::new(SliceLexer::new(&bytes, false), limits);
            let mut reader = PullParser::from_reader(
                &bytes[..],
                BufConfig {
                    read_buf: size,
                    ..BufConfig::default()
                },
                limits,
            );
            loop {
                let event = reader.next_event();
                assert_eq!(event, reference.next_event(), "{name}, buffer {size}");
                match event {
                    Ok(Event::Eof) => {
                        assert!(expected, "{name}");
                        break;
                    }
                    Err(error) => {
                        assert_eq!(Err(error), result, "{name}");
                        break;
                    }
                    Ok(Event::Key { .. } | Event::Str { .. } | Event::Num) => assert_eq!(
                        reader.content(),
                        reference.content(),
                        "{name}, buffer {size}"
                    ),
                    Ok(_) => {}
                }
            }
        }
        seen += 1;
    }
    assert_eq!(seen, 318, "incomplete or different JSONTestSuite revision");
    assert!(
        choices.is_empty(),
        "unused implementation-defined decisions"
    );
}
