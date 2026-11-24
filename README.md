# rs-json 0.2 development

The slice parser builds a flat tape with 16-byte nodes. `parse` accepts a string,
`parse_bytes` accepts UTF-8 bytes, and `Parser::parse` reuses its buffers across
documents. `PullParser` exposes the same grammar for byte slices and synchronous
readers. The published 0.1 API is on [main](https://github.com/codixor/rs-json/tree/main).

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

The following features and performance targets are described in the
[0.2 roadmap](docs/PLAN.md). File mapping and
the higher-level compatibility features are still in development.

| Feature | Default | Adds |
|---------|---------|------|
| `std` | ✓ | std I/O, file helpers |
| `simd` / `simd-dyn` | ✓ | `wide` kernels / x86 runtime AVX2 dispatch |
| `read` | | `io::Read` streaming lexer |
| `mmap` | | memory-mapped files |
| `ndjson` | | line splitting (`memchr`) |
| `pointer` | | streaming JSON Pointer |
| `serde` / `ld` | | serde_json facade / JSON-LD + Google builders |
| `legacy` | | old `JsonItem` API shim |

See [the parser analysis](docs/ANALYSIS.md) for the design rationale.

License: MIT OR Apache-2.0.
