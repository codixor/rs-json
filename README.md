# rs-json 0.2

The slice parser builds a flat tape with 16-byte nodes. `parse` accepts a string,
`parse_bytes` accepts UTF-8 bytes, and `Parser::parse` reuses its buffers across
documents. `PullParser` exposes the same grammar for byte slices and synchronous
readers. Install with `cargo add rs-json@0.2` (Rust 1.100 or newer).

### Nesting limit

Each open array or object counts as one level, including an empty container.
Scalars have depth zero. The default allows 512 levels. On the first opener
that exceeds the limit, parsing returns `ErrorKind::DepthLimit` at its byte
offset. Skipping a value applies the same check.

```rust
use rs_json::{ErrorKind, Parser};

let mut parser = Parser::new();
parser.limits.max_depth = 1;
assert!(parser.parse("[1, 2]").is_ok());
let error = parser.parse("[{}]").unwrap_err();
assert_eq!(error.kind, ErrorKind::DepthLimit);
assert_eq!(error.offset(), 1);
```

### Parsing a reader

Enable the `read` feature to use `PullParser::from_reader` with a `std::io::Read`
source. The parser borrows token bytes from its refill buffer and retains a
token that spans multiple reads. `Limits::max_token_bytes` bounds the raw
string body or number in bytes. Both limits also apply to skipped values.
Use `multi_document(true)` for successive JSON documents.

```rust
use rs_json::{Limits, config::BufConfig, stream::{Event, PullParser}};

fn main() -> Result<(), rs_json::Error> {
    let input = &b"{\"page\":1} {\"page\":2}"[..];
    let mut parser = PullParser::from_reader(input, BufConfig::default(), Limits::default())
        .multi_document(true);
    for _ in 0..2 {
        parser.skip_value()?;
        assert_eq!(parser.next_event()?, Event::End);
    }
    assert_eq!(parser.next_event()?, Event::Eof);
    Ok(())
}
```

### Optional APIs and migration from 0.1

| Feature | API |
|---------|-----|
| `std` (default) | `input::parse_file` returns owned file bytes. Call `.document()?` to parse. |
| `simd` (default) | Portable 16-byte string scans. `simd-dyn` enables this path with `std`. |
| `read` | Synchronous, bounded-buffer reader events. |
| `mmap` | Large ordinary files use anonymous mapped snapshots. `unsafe map_immutable_file` provides zero-copy input when the caller guarantees immutability. |
| `ndjson` | `lines` and `for_each_document` for newline-delimited records. |
| `pointer` | `stream::pointer::PointerSet::compile` and `.select` for streaming RFC 6901 matches. |
| `serde` | The 0.1 typed codec functions and `Value::to_serde`. |
| `ld` | The 0.1 JSON-LD document, array, object and Google profile builders. Enables `serde`. |
| `legacy` | Import `rs_json::legacy::parse` to keep 0.1 indexing, raw string bodies and missing-value behavior during migration. |

The 0.2 root `parse` returns a `Document`. Use `doc.root().get("name")` for
fields, `.at(i)` for array elements and `.raw()` for the original token text.
String `.as_str()` returns decoded text. Errors provide a byte offset, line
and column. New code can use these diagnostics without storing source text.

The [benchmark report](https://github.com/codixor/rs-json/blob/v0.2.0/docs/BENCH.md)
includes one-shot and reused parser results, reader peak RSS and corpus comparisons.

Licensed under MIT OR Apache-2.0.
