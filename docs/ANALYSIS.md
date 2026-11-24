# rs-json review and parser analysis

Reviewed: `codixor/rust-misc` @ `ae68abb` (`rs-json/`), `bufjson 1.3.0`,
`aidenaistar/json_fast` (HEAD), toolchain `rustc 1.100.0 (2026-11-09)`.

## 1. Current rs-json (rust-misc)

Size: parser core ~680 lines, `ld` JSON-LD builders ~6,500 lines,
serde facade 140 lines, 14 test files, 3 Criterion benches.

### What is good and should be kept
- Zero-copy model, lazy numbers, typed `JsonErrorKind`, golden tests.
- Disciplined PLAN with measured-adoption rule (memchr was benchmarked and
  rejected because it regressed short strings — this finding shapes the new
  scan design: scalar prefix first, wide kernels only for long runs).
- The JSON-LD layer is well tested; it moves over unchanged behind `ld`.

### Problems (verified on 1.100.0 unless noted)
| # | Problem | Evidence / impact |
|---|---------|-------------------|
| P1 | Recursive descent without a depth limit | Deeply nested untrusted input can exhaust the call stack. Use an iterative parser with an explicit nesting limit. |
| P2 | Escaped keys never match | `parse(r#"{"ab":1}"#)["ab"]` → not found; key compared as raw `ab`. |
| P3 | No unescape API | `as_str()` returns raw text (`"x\ny"` → `x\\ny`); callers must unescape themselves. |
| P4 | Lone surrogates accepted | `"\ud800"` parses OK; serde_json rejects it. |
| P5 | Allocation-heavy DOM | `JsonItem` = **56 bytes**; one heap `Vec` (cap 4) per non-empty container, grown by doubling; recursive tree → poor locality. |
| P6 | O(n) key lookup per access, no skip pointers | `Index<&str>` scans entries linearly every call. |
| P7 | Error path allocates | every `JsonError` builds a `String` extract; no line/column. Slows the "reject invalid" lane. |
| P8 | `&str` input only | no `&[u8]`, `io::Read`, file, mmap, NDJSON or streaming → large files must be fully loaded and fully materialised. |
| P9 | Hard deps for parse-only users | `serde` + `serde_json` always compiled; `ld` (6.5k lines) always compiled. |
| P10 | Metadata | `repository = github.com/mwdghaz/rust-misc` (not codixor); toolchain pinned 1.95.0. |

## 2. bufjson 1.3.0 — what to adopt

bufjson is a streaming lexer + pull parser + streaming JSON Pointer, no DOM.
Its own comparison claims ~1.35× serde_json throughput on single-threaded
parsing, and concedes simd-json is ~1.74× faster than bufjson on pure parsing.

| Technique | Where in bufjson | Adopt? | Where in new crate |
|-----------|------------------|--------|--------------------|
| Defer all work: no unescape / number conversion in the lexer; `escaped` flag per string token | `lexical::Content::is_escaped`, `Unescaped::{Literal,Expanded}` | **Yes** | `lex::Token::Str{escaped}`, `Node::ESCAPED`, `Value::as_str -> Cow` |
| 256-entry cache-aligned "boring byte" LUT | `lexical/state.rs` `BORING` | **Yes** | `scan/scalar.rs` |
| `wide` u8x16/u8x32 string kernel; runtime AVX2/SSE2 dispatch once on x86, compile-time elsewhere; scalar tail, no padding | `state.rs` `simd`, `simd_dyn` | **Yes** (+ SWAR middle tier) | `scan/simd.rs`, `scan/swar.rs` |
| Three analyzers: fixed buffer, `io::Read` w/ 8 KiB buffers, async pipe; cross-boundary tokens copied | `lexical/{fixed,read,pipe}.rs` | **Yes** (fixed + read; async later) | `lex/slice.rs`, `lex/read.rs` |
| Bit-packed nesting context (bitvec) with inline small storage | `syntax.rs` `StructContext` | **Yes**, without the bitvec dep | `stack.rs` (2×u64 inline + Vec spill) |
| Own integer parsers (faster than core); f64 forwards to core | `lexical::parse_i64/u64/f64` | **Yes** | `num.rs` |
| Streaming JSON Pointer evaluation | `pointer/` | **Yes** (feature) | `stream/pointer.rs` |
| Precise line/column on every token | `Pos` everywhere | **No** — lazy: offset only, line/col resolved on error; streaming counts newlines per chunk | `error.rs` |
| `no_std` (+alloc) | crate features | **Yes** | `#![no_std]` without `std` feature |

What bufjson does **not** give us: an in-memory DOM. For whole-document
parsing the targets are sonic-rs / simd-json, which is why the new crate
adds a flat tape DOM (simd-json-style) on top of the same lexer.

## 3. json_fast — nothing to adopt

`json_fast::parse` matches the input against **12 hard-coded byte strings**
(`{"ok":true}`, `{"status": false}`, …) and returns a cloned pre-built
value; everything else is forwarded to `serde_json` and converted, and
**arrays are silently turned into `null`** (data loss). The `regex`
dependency is unused. Its "35% faster than serde_json" number is the
benchmark input hitting the lookup table.

Lesson kept: benchmark inputs in `benches/parse_small.rs` are varied and
the differential fuzzer compares results with serde_json, so no
special-casing can pass as a speed-up.

## 4. Performance design for small and large inputs

```
              ┌──────────── scan (LUT → SWAR → SIMD, scalar tail) ───────────┐
              │                                                              │
  &str/&[u8] ─┤ SliceLexer ──► tape builder (iterative) ──► Document/Value    │
              │            └─► PullParser ──► events / skip / JSON Pointer    │
  io::Read  ──┤ ReadLexer (refill buf + spill) ──► PullParser                 │
  file      ──┘ input::parse_file: ≤1 MiB read→Vec · >8 MiB mmap · else Vec   │
```

**Small documents (≤ 4 KiB, the API/websocket case)** — cost is dominated by
allocation and per-token overhead, not scanning:
- `Parser` reuse: tape, stack and open-index `Vec`s kept across parses →
  **0 allocations per document after warm-up** (asserted with a counting
  allocator).
- One growable allocation even in one-shot mode (vs. one per container).
- Scalar prefix of 16 bytes before any wide kernel (most keys and short
  values end inside it — the reason memchr lost before).
- Literal `true/false/null` checked as one 4-byte word compare.

**Medium documents (64 KiB – 64 MiB)** — scanning throughput matters:
- SIMD string runs (twitter.json is string/unicode heavy).
- SWAR whitespace skipping for pretty-printed input.
- `&str` input skips UTF-8 validation entirely; `&[u8]` validates only
  non-ASCII runs inside strings with `simdutf8`.
- Tape capacity pre-sized from input length (start `len / 8`, tune).
- Research gate G5: simdjson-style stage-1 structural indexing (64-byte
  blocks, quote/backslash masks, portable prefix-XOR) only if it beats the
  single-pass lexer by ≥ 20% on citm/twitter.

**Large files (≥ 64 MiB, up to unbounded)**:
- DOM path: mmap (feature `mmap`) → zero copy, page-cache backed; tape
  offsets are `u32`, so DOM is limited to 4 GiB.
- Streaming path: `ReadLexer` with a fixed buffer (sweep 8/16/64/256 KiB;
  bufjson default is 8 KiB) + `PullParser::skip_value` → memory stays
  ≈ buffer size regardless of file size; JSON Pointer selection skips
  unwanted subtrees without materialising them.
- NDJSON: `memchr` line split + `Parser` reuse per line.

## 5. Dependency versions (crates.io, 2026-11-24)

| Crate | Old rs-json | New |
|-------|-------------|-----|
| rust toolchain | 1.95.0 | **1.100.0** |
| serde | 1.0.228 | 1.0.229 (optional) |
| serde_json | 1.0.149 | 1.0.154 (optional) |
| thiserror | 2.0.18 | 2.0.21 (`default-features = false`) |
| criterion | 0.8.2 | 0.8.2 |
| simd-json (dev) | 0.17.0 | 0.19.0 |
| sonic-rs (dev) | 0.5.8 | 0.5.12 |
| memchr | (rejected) | 2.8.3 (optional, `ndjson` only) |
| wide | — | 1.7.1 (optional, `simd`) |
| simdutf8 | — | 0.1.5 |
| memmap2 | — | 0.9.12 (optional, `mmap`) |
| tokio | — | 1.54.1 (optional, `async`) |
| bufjson (dev, comparator) | — | 1.3.0 |
| proptest (dev) | — | 1.12.0 |
| legacy `json` 0.12.4, ahash, hashbrown, indexmap (dev) | present | dropped (unused by new benches) |
