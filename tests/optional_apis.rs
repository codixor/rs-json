//! Release coverage for file storage, streaming selection, NDJSON and Serde.

#[cfg(all(feature = "pointer", feature = "read"))]
#[test]
fn pointers_emit_full_values_and_validate_skipped_branches() {
    use rs_json::{
        Limits,
        config::BufConfig,
        stream::{Event, PullParser, pointer::PointerSet},
    };
    let pointers =
        PointerSet::compile(["", "/a~1b/0", "/a~1b/0/~0", "/a~1b/00", "/", "/a~1b/0/~0"]).unwrap();
    for size in [1, 7, 64, 8192] {
        let input = br#"{"a\u002fb":[{"~":42}],"":"x","unused":[{"aa":"b"}]}"#;
        let mut parser = PullParser::from_reader(
            input.as_slice(),
            BufConfig {
                read_buf: size,
                ..BufConfig::default()
            },
            Limits::default(),
        );
        let mut hits = vec![Vec::new(); 6];
        pointers
            .select(&mut parser, |id, event, bytes| {
                hits.get_mut(id).unwrap().push((event, bytes.to_vec()));
            })
            .unwrap();
        assert_eq!(hits.first().unwrap().first().unwrap().0, Event::ObjectStart);
        assert_eq!(hits.first().unwrap().last().unwrap().0, Event::ObjectEnd);
        assert_eq!(
            hits.get(1).unwrap(),
            &vec![
                (Event::ObjectStart, vec![]),
                (Event::Key { escaped: false }, b"~".to_vec()),
                (Event::Num, b"42".to_vec()),
                (Event::ObjectEnd, vec![])
            ]
        );
        assert_eq!(hits.get(2).unwrap(), &vec![(Event::Num, b"42".to_vec())]);
        assert_eq!(hits.get(2), hits.get(5));
        assert_eq!(hits.get(3).unwrap(), &Vec::new());
        assert_eq!(
            hits.get(4).unwrap(),
            &vec![(Event::Str { escaped: false }, b"x".to_vec())]
        );
    }
    for p in ["abc", "/a~", "/~9"] {
        assert!(PointerSet::compile([p]).is_err());
    }
    let mut parser = PullParser::from_reader(
        &br#"{"read":1,"ignore":[0,]}"#[..],
        BufConfig::default(),
        Limits::default(),
    );
    assert!(
        PointerSet::compile(["/read"])
            .unwrap()
            .select(&mut parser, |_, _, _| {})
            .is_err()
    );
}

#[cfg(feature = "ndjson")]
#[test]
fn ndjson_reuses_the_parser_and_reports_absolute_positions() {
    let mut parser = rs_json::Parser::new();
    let src = b"\n  \t\r\n1\r\n[2]\n\x0b\n";
    let mut seen = Vec::new();
    let err =
        rs_json::ndjson::for_each_document(&mut parser, src, |d| seen.push(d.src)).unwrap_err();
    assert_eq!(seen, vec![b"1".as_slice(), b"[2]"]);
    assert_eq!(
        err.pos,
        rs_json::Pos {
            offset: 13,
            line: 5,
            col: 1
        }
    );
    let err = rs_json::ndjson::for_each_document(&mut parser, b"0\n\"\xff\"", |_| {}).unwrap_err();
    assert_eq!(
        err.pos,
        rs_json::Pos {
            offset: 3,
            line: 2,
            col: 2
        }
    );
}

#[cfg(feature = "serde")]
#[test]
fn tape_conversion_decodes_keys_and_preserves_numeric_and_duplicate_semantics() {
    let src = r#"{"a":0,"\u0061":[true,{"b":null},-0.0,18446744073709551615],"s":"\u2603"}"#;
    let doc = rs_json::parse(src).unwrap();
    assert_eq!(
        doc.root().to_serde().unwrap(),
        rs_json::from_str::<rs_json::JsonValue>(src).unwrap()
    );
    assert!(rs_json::parse("1e9999").unwrap().root().to_serde().is_err());
}

#[cfg(feature = "mmap")]
#[test]
fn file_snapshots_survive_truncation() {
    use rs_json::{
        config::BufConfig,
        input::{Storage, map_immutable_file, parse_file},
    };
    use std::{fs, io::Write};
    let dir = std::env::temp_dir().join(format!("rs-json-snapshot-{}", std::process::id()));
    fs::create_dir(&dir).unwrap();
    let path = dir.join("record.json");
    let mut file = fs::File::create_new(&path).unwrap();
    file.write_all(b"[42]").unwrap();
    drop(file);
    let cfg = BufConfig {
        small_file_max: 0,
        mmap_min: 1,
        ..BufConfig::default()
    };
    let snapshot = parse_file(&path, cfg).unwrap();
    assert!(matches!(snapshot.storage, Storage::Mmap(_)));
    {
        #[allow(
            unsafe_code,
            reason = "test owns the directory and drops the mapping before changing the file"
        )]
        // SAFETY: this private test file is immutable for the lifetime of this mapping.
        let mapped = unsafe { map_immutable_file(&path) }.unwrap();
        assert_eq!(mapped.bytes(), b"[42]");
    }
    fs::write(&path, []).unwrap();
    let empty = parse_file(&path, cfg).unwrap();
    assert_eq!(empty.bytes(), b"");
    fs::remove_file(path).unwrap();
    fs::remove_dir(dir).unwrap();
    assert_eq!(
        snapshot.document().unwrap().root().at(0).unwrap().as_i64(),
        Some(42)
    );
}
