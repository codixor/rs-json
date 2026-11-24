# rs-json 0.2 plan — stages and gates

Every stage ends green on: `cargo fmt --check`, `cargo clippy --all-targets
--all-features -- -D warnings`, `cargo test --all-features`,
`cargo test --no-default-features`, `cargo check --benches --all-features`.
Benchmark numbers go into `docs/BENCH.md` with CPU model, OS, toolchain and
`RUSTFLAGS` (default and `-C target-cpu=native` reported separately).

| Stage | Scope | Gate |
|-------|-------|------|
| S0 | Repo bootstrap: this skeleton, LICENSE-MIT/APACHE, corpus scripts run | CI green with stubs `#[ignore]`d |
| S1 | `SliceLexer` complete (strings incl. surrogate pairs, numbers, literals, UTF-8 for `&[u8]`) | JSONTestSuite y/n all correct through the lexer + `PullParser<SliceLexer>` |
| S2 | Iterative tape builder, `Document`, `Value`, `Parser` reuse, `unescape`, `unescaped_eq` | **G1**: `tests/limits.rs` un-ignored and passing; 0 allocs/parse after warm-up; ≥ 2× old rs-json on `small/*` and `nested` |
| S3 | `ReadLexer` + spill + per-chunk newline count; `PullParser::skip_value` | **G2**: conformance identical for buffer sizes 1, 7, 64, 8192; RSS on 1 GiB file ≤ 4× `read_buf` + 8 MiB |
| S4 | Scan tuning: AVX2 dispatch (`simd-dyn`), SCALAR_PREFIX sweep, SWAR int parse, tape capacity heuristic | **G3**: within 1.3× of sonic-rs on twitter/citm (DOM); ≥ bufjson on streaming at best buffer size |
| S5 | `input::parse_file` (read / mmap), `ndjson`, `pointer` | **G4**: file thresholds chosen from `stream_large` data, recorded in BENCH.md |
| S6 | Optional research: stage-1 structural index for docs ≥ 64 KiB | **G5**: adopt only if ≥ 20% faster on citm + twitter with no small-doc regression |
| S7 | Move `ld` + `serde_codec` from rust-misc; `legacy` shim; migration notes | all old rs-json tests/goldens pass under `--features ld,serde,legacy` |
| S8 | Fuzz (differential vs serde_json, 1 h clean), miri on `scan`/`stack`, docs, README, publish dry-run | `cargo publish --dry-run` clean |

Adoption rule kept from the old PLAN: a helper crate or unsafe kernel is
merged only with a benchmark showing a win on the kept workload mix and no
regression > 3% elsewhere.
