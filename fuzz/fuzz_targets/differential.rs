//! Differentially check byte, string and reader parsing against serde_json.
#![no_main]
use libfuzzer_sys::fuzz_target;
use rs_json::{
    Limits,
    config::BufConfig,
    stream::{Event, PullParser},
};

fuzz_target!(|data: &[u8]| {
    let bytes = rs_json::parse_bytes(data);
    if let Ok(text) = std::str::from_utf8(data) {
        let text_doc = rs_json::parse(text);
        assert_eq!(text_doc.as_ref().map(|_| ()), bytes.as_ref().map(|_| ()));
        if let (Ok(a), Ok(b)) = (&text_doc, &bytes) {
            assert_eq!(a.root().to_serde(), b.root().to_serde());
        }
    }
    let mut reader = PullParser::from_reader(
        data,
        BufConfig {
            read_buf: [1, 7, 64, 8192][data.len() % 4],
            ..BufConfig::default()
        },
        Limits::default(),
    );
    let accepted = loop {
        match reader.next_event() {
            Ok(Event::Eof) => break true,
            Err(_) => break false,
            Ok(_) => {}
        }
    };
    assert_eq!(accepted, bytes.is_ok());
    match serde_json::from_slice::<serde_json::Value>(data) {
        Ok(expected) => assert_eq!(bytes.unwrap().root().to_serde().unwrap(), expected),
        Err(err) => {
            // A DOM number can preserve a spelling outside f64's range.
            // Serde's independent default nesting limit is also lower.
            let msg = err.to_string();
            if !msg.starts_with("number out of range")
                && !msg.starts_with("recursion limit exceeded")
            {
                assert!(bytes.is_err(), "{err}");
            }
        }
    }
});
