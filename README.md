# rs-json 0.2 skeleton

This branch stages the proposed 0.2 API and development plan. Several parser entry points are placeholders. The [main branch](https://github.com/codixor/rs-json/tree/main) contains the working 0.1 crate. The supplied workflow is staged for review in [docs/ci-draft.yml](docs/ci-draft.yml).

Zero-copy JSON for Rust 1.100+, one lexer core for every input size:

- **Tape DOM** — `rs_json::parse(&str)` / reusable `Parser` (zero
  allocations per document after warm-up), 16-byte nodes, O(1) subtree skip.
- **Streaming** — `PullParser` over a slice or any `io::Read` with a fixed
  buffer; constant memory on multi-GB files; `skip_value` and streaming JSON
  Pointer selection.
- **Files** — `input::parse_file` picks read-to-buffer, mmap or streaming
  by size. NDJSON helper.
- Strings stay borrowed unless they contain escapes; numbers convert on
  access. SIMD string scanning via `wide` (SSE2/AVX2/NEON/simd128), SWAR and
  scalar fallbacks, no input padding, `no_std + alloc`.

```rust
let doc = rs_json::parse(r#"{"s":"BTCUSDT","p":"94210.10","q":[1,2,3]}"#)?;
let root = doc.root();
assert_eq!(root.get("s").unwrap().as_str().unwrap(), "BTCUSDT");
```

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

Status: 0.2 skeleton — see `docs/PLAN.md`. Design rationale and review of
the previous version: `docs/ANALYSIS.md`.

License: MIT OR Apache-2.0.
