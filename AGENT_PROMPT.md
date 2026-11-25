# Agent prompt — build `rs-json` 0.2 as an independent crate

You are implementing **rs-json 0.2**, a standalone Rust crate (repo
`codixor/rs-json`) extracted from `codixor/rust-misc/rs-json`. This
directory is the starting skeleton: it compiles on Rust 1.100.0, the module
layout and public API are fixed, and every `todo!()` marks work for you.

Read first, in order: `docs/ANALYSIS.md` (why), `docs/PLAN.md` (stages and
gates), then `src/lib.rs` and each module's doc comments (the algorithms are
specified there).

## Hard constraints

1. **Toolchain**: Rust **1.100.0**, edition 2024, `rust-version = "1.100"`
   (pinned in `rust-toolchain.toml`). Use current stable features freely.
2. **Dependencies**: only those already in `Cargo.toml`, at those versions
   (checked on crates.io 2026-11-24). Before adding or bumping anything, run
   `cargo info <crate>` and write the reason plus a benchmark in
   `docs/BENCH.md`. Do **not** add `fast-float2`, `lexical-core`, `bitvec`,
   `smallvec`, `bumpalo` or a hashing crate unless a gate fails without it.
3. **Unsafe**: `unsafe_code = "deny"`. The only allowed exceptions, each in
   its own small function with `#[allow(unsafe_code)]` and a `// SAFETY:`
   comment: (a) calling `#[target_feature(enable = "avx2")]` kernels after
   `is_x86_feature_detected!`, (b) `memmap2::Mmap::map`. Nothing else —
   no `get_unchecked`, no `from_utf8_unchecked` (use the validated-once
   invariant + `from_utf8` on the slow path, or keep `&str` slices from the
   `&str` input).
4. **No recursion** anywhere in parsing, skipping, unescaping or dropping.
   Depth is bounded only by `Limits::max_depth`.
5. **Zero-copy**: no input padding, no input copy for `&str`/`&[u8]`/mmap.
   Strings without escapes and all numbers are returned as borrowed slices.
6. **Errors** never allocate (`Error` is `Copy`).
7. Keep the crate `no_std + alloc` without the `std` feature.
8. Every stage ends with all of:
   ```
   cargo fmt --check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --all-features
   cargo test --no-default-features
   cargo check --benches --all-features
   ```
   Do not mark a stage done on reasoning alone; paste the command output
   summary and bench numbers into your stage report.

## Work order

Follow `docs/PLAN.md` S0 → S8. Summary of what each module needs:

| Module | Implement |
|--------|-----------|
| `scan` | Already working (LUT, SWAR, `wide` u8x16, property test). S4: add AVX2 `u8x32` + runtime dispatch cached in `std::sync::OnceLock<fn(&[u8], usize) -> usize>` under `simd-dyn`; compile-time choice otherwise; NEON via `wide` on aarch64; sweep `SCALAR_PREFIX` ∈ {8,16,32}. Test each kernel against the naive scan. |
| `lex::slice` | Strings (escapes, `\uXXXX`, surrogate pairs → `LoneSurrogate`, control bytes, UTF-8 for untrusted input via `simdutf8` on non-ASCII runs only), numbers (port old grammar, set `NUM_INT` hint), literals via 4-byte word compare. |
| `lex::read` | Refill/compact buffer, spill tokens across refills (cap `max_token_bytes` → `SizeLimit`), retry `Interrupted`, per-chunk newline counting for `Pos`, `into_inner`. Must produce the identical token stream as `SliceLexer` for any buffer size ≥ 1. |
| `dom` | Iterative `build` with a 4-state grammar enum, `NestStack`, `open: Vec<u32>` patching `b` (count) and `c` (end index). Tape reserve heuristic. `Parser` reuse path with zero steady-state allocation. Keep `Node` at 16 bytes. Make `Value::raw` infallible by construction. |
| `unescape` | Bulk-copy plain runs with `scan::string_run_end`; `unescaped_eq` streaming compare without allocation. |
| `stream` | `PullParser::next_event` (same grammar as `dom::build`), `multi_document`, `skip_value` (depth counter only), `pointer::PointerSet` trie + selective extraction API. |
| `input` | `parse_file` strategy table in the module doc; mmap behind `mmap`; thresholds from benches. |
| `ndjson` | done (splitter); add `for_each_document(&mut Parser, buf, f)` helper. |
| `serde_codec`, `ld` | Move from `rust-misc/rs-json` unchanged (S7), including its tests, goldens, 16 examples and `json_ld` bench. Gate them behind features; parse-only builds must not compile serde. |
| `legacy` | `#[deprecated]` `JsonItem` shim over `dom::Value` with the old method names so rust-misc callers migrate gradually. |

## Tests you must add

- `tests/conformance.rs`: JSONTestSuite through **every** entry point;
  `ReadLexer` with buffer sizes 1, 7, 64, 8192; record `i_*` decisions in
  `tests/conformance/decisions.toml`.
- Port all old `rust-misc/rs-json/tests/*.rs` and `tests/golden/` (adapt to
  the new API; old behaviours that were bugs — P2, P3, P4 in ANALYSIS.md —
  get new expected values and a comment).
- `tests/alloc_count.rs`: counting `#[global_allocator]`; assert 0
  allocations for `Parser::parse` after warm-up on every `parse_small` sample.
- proptest: random JSON values → serialize with serde_json → parse with every
  entry point → compare to `serde_json::Value`.
- Un-ignore `tests/limits.rs` at S2.
- `fuzz/` differential target: run ≥ 1 h clean before S8.

## Benchmarks you must run and report

`scripts/fetch-corpus.sh`, `scripts/gen-large.sh`, then
`cargo bench --bench parse_small`, `parse_corpus`, `stream_large`.
Enable the commented `rs_json` lanes as features land. Report in
`docs/BENCH.md` a table per bench: MB/s and ns/doc for rs-json, old rs-json
(git dep on rust-misc `ae68abb`), serde_json, sonic-rs, simd-json, bufjson;
peak RSS for `stream_large`. Run twice: default flags and
`RUSTFLAGS="-C target-cpu=native"`. Never tune on one input — a change is
kept only if it wins on the mix (small + corpus + large) per the adoption
rule in PLAN.md.

## Out of scope

Serialization/writer (beyond the serde facade), JSON5/comments, async
(`async` feature stays a stub unless asked), parallel parsing.

## Deliverable per stage

A short report: what changed, commands run with pass/fail, bench table
deltas, open questions. Stop and ask if a gate cannot be met, rather than
weakening the gate or the tests.
