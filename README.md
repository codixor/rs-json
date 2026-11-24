# rs-json 0.2 development

The slice parser builds a flat tape with 16-byte nodes. `parse` accepts a string,
`parse_bytes` accepts UTF-8 bytes, and `Parser::parse` reuses its buffers across
documents. A slice-backed `PullParser` exposes the same grammar as a stream of
events. The published 0.1 API is on [main](https://github.com/codixor/rs-json/tree/main).

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

The following features and performance targets are described in the
[0.2 roadmap](docs/PLAN.md). Buffered reader parsing, file mapping and
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
