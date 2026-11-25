# rs-json 0.4

The slice parser builds a flat tape with 16-byte nodes. `parse` accepts a string,
`parse_bytes` accepts UTF-8 bytes, and `Parser::parse` reuses its buffers across
documents. `PullParser` exposes the same grammar for byte slices and synchronous
readers. Install with `cargo add rs-json@0.4` (Rust 1.100 or newer).

### Shared parser limits

Version 0.4 re-exports `rs_format::Limits` and adopts its inclusive defaults.
Use `parse_with_limits`, `parse_bytes_with_limits`, `Parser::limits` or the
limits parameter on `PullParser`. With the `serde` feature, use
`from_str_with_limits`, `from_slice_with_limits` or `from_reader_with_limits`.
Their convenience counterparts use `Limits::default()`.

| Field | Default | JSON accounting |
| --- | ---: | --- |
| `max_depth` | 128 | Open objects and arrays. A scalar has depth zero. An empty root container has depth one. |
| `max_document_bytes` | 16 MiB | Encoded input bytes, including whitespace and delimiters. |
| `max_token_bytes` | 1 MiB | Encoded bytes of a string or key body, number, or `true`, `false` and `null` literal. Quotes are excluded and escape sequences count in full. |
| `max_expansion` | 32 Mi units | JSON has no expansion constructs. Escapes decode to the same or fewer bytes, so every document costs zero units. |

The native parser can use larger custom limits. Its tape stores 32-bit offsets
and individual token spans use 32-bit lengths. The typed text decoder uses
serde_json with an inclusive hard ceiling of 128 open containers to bound its
call stack. A request for a larger typed depth still reports `Category::Limit`
at the 129th opener. A `from_value` conversion starts with an already-owned
value and has no encoded input to measure. The file helper takes a snapshot
before parsing. Use `FileDocument::document_with_limits` on that snapshot,
or a bounded `PullParser` to validate a file incrementally.

In 0.3 the default native depth was 512, the DOM document cap was `u32::MAX`
bytes, and reader strings and numbers allowed 64 MiB. Existing `Limits`
struct literals can use `..Limits::default()` to fill the new expansion field.

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
token that spans multiple reads. Skipped values obey the same limits.
With `multi_document(true)` each root starts a fresh byte budget immediately
after the previous root. Separator whitespace counts toward the next budget.
Positions stay relative to the beginning of the stream.

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

### Optional APIs and migration

| Feature | API |
|---------|-----|
| `std` (default) | `input::parse_file` returns owned file bytes. Call `.document()?` to parse. |
| `simd` (default) | Portable 16-byte string scans. `simd-dyn` enables this path with `std`. |
| `read` | Synchronous, bounded-buffer reader events. |
| `mmap` | Enables [Codixor's rs-mmap](https://crates.io/crates/rs-mmap). On Unix and Windows, large ordinary files use anonymous mapped snapshots. `unsafe map_immutable_file` provides zero-copy input when the caller guarantees immutability. |
| `ndjson` | `lines` and `for_each_document` for newline-delimited records. |
| `pointer` | `stream::pointer::PointerSet::compile` and `.select` for streaming RFC 6901 matches. |
| `serde` | Typed encoding and decoding through `serde_json`, plus `Value::to_serde` for converting a native tape value. |
| `ld` | The 0.1 JSON-LD document, array, object and Google profile builders. Enables `serde`. |
| `legacy` | Import `rs_json::legacy::parse` to keep 0.1 indexing, raw string bodies and missing-value behavior during migration. |

Since 0.2, the root `parse` function returns a `Document`. Use `doc.root().get("name")` for
fields, `.at(i)` for array elements and `.raw()` for the original token text.
String `.as_str()` returns decoded text. Errors provide a byte offset, line
and column. New code can use these diagnostics without storing source text.

Use `FormatError` for format-independent handling of native, typed and legacy
errors. It exposes a `Category` and the known coordinates in a `Position`.
Errors discard source text and custom Serde and I/O messages before callers
can format or traverse them. Native failures and typed limit failures carry
byte offsets and one-based byte columns. Typed backend failures retain the
backend's known line and column. An unavailable coordinate is `None`.

```rust
use rs_json::{Category, FormatError, Limits, parse_with_limits};

let limits = Limits { max_token_bytes: 2, ..Limits::default() };
let error = parse_with_limits(r#"{"long":0}"#, limits).unwrap_err();
let diagnostic: &dyn FormatError = &error;
assert_eq!(diagnostic.category(), Category::Limit);
assert_eq!(diagnostic.position().unwrap().offset, Some(4));
```

The existing `JsonSerdeError::category()` and `position()` accessors retain
their backend-specific return types. Use `FormatError::category(&error)` to
include encoding and limit failures. Code matching `Serialize { message }` can
now match `Serialize { category }`. The legacy extract accessors return `None`.

The [0.4 benchmark results](docs/BENCH.md#04-shared-limits-on-arbi-pc) compare the
limited native slice parser with 0.3.0 on eight fixtures.

The [0.2.0 benchmark report](https://github.com/codixor/rs-json/blob/v0.2.0/docs/BENCH.md)
includes one-shot and reused parser results, reader peak RSS and corpus comparisons.

Licensed under MIT OR Apache-2.0.
