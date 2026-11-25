# 0.2 slice parser measurements

The tables and reproduction commands below use the [historical v0.2 harness](https://github.com/codixor/rs-json/tree/v0.2.0/benches) and its [0.1.0 comparison results](https://github.com/codixor/rs-json/blob/v0.2.0/docs/BENCH.md). Starting in
0.4, the published benchmark dependency is rs-json 0.3.0 and its Criterion
lane is `rs_json_030::parse`. The generated mixed fixture measures 16,804,466
bytes. The 0.4 corpus harness permits that exact document size.

The [corpus benchmarks](#corpus-benchmarks-on-arbi-pc) compare seven operations on
four in-memory documents in default and native Ryzen builds.

The first implementation of the nesting limit enables the two `rs-json` lanes
in `parse_small`. These measurements establish a baseline for the development
branch. The staged skeleton had no working DOM parser to time.

Criterion quick-run point estimates on Windows 11 x64, Intel Core i9-9820X,
Rust 1.100.0. The run used the default CPU target with no `RUSTFLAGS`, one
Cargo build job, and the optimized bench profile with LTO and debug symbols
disabled. Each sample fits in 200 bytes. Parser reuse retains the tape and
container stack between iterations.

| Sample | One-shot ns/doc | Reused ns/doc | One-shot MiB/s | Reused MiB/s |
| --- | ---: | ---: | ---: | ---: |
| Tiny object | 190 | 83 | 55 | 126 |
| Ticker | 1,191 | 627 | 135 | 257 |
| Escaped strings | 601 | 179 | 116 | 389 |
| Nested object | 1,825 | 1,161 | 103 | 163 |

```sh
CARGO_BUILD_JOBS=1 CARGO_PROFILE_BENCH_LTO=false CARGO_PROFILE_BENCH_DEBUG=0 \
  cargo bench --locked --bench parse_small -- 'rs_json::' --quick --noplot
```

The Ryzen measurements below compare both versions on these same fixtures.
The latest results include the [one-shot escaped-string comparison](#one-shot-escaped-strings-on-arbi-pc).

## Reader refill sizes

The reader consumes the same 12,545-byte JSON array of 256 records at each buffer
size. Each iteration creates a reader and consumes every event through EOF.
The input is memory-backed. These times measure tokenization and refill
handling. The 1 GiB run below measures peak resident memory separately.

Windows 11 x64, Intel Core i9-9820X, Rust 1.100.0, default CPU target.
The optimized quick run used one build job with LTO and debug symbols disabled.

| Read buffer (bytes) | µs/document | MiB/s |
| ---: | ---: | ---: |
| 1 | 241.64 | 49.51 |
| 7 | 132.03 | 90.61 |
| 64 | 106.40 | 112.44 |
| 8192 | 100.45 | 119.10 |

```sh
CARGO_BUILD_JOBS=1 CARGO_PROFILE_BENCH_LTO=false CARGO_PROFILE_BENCH_DEBUG=0 \
  cargo bench --locked --features read --bench stream_large -- reader_buffers --quick --noplot
```

## 1 GiB streaming reader RSS

At commit `20bcd689d2bead361f94fda74ce48620a1a3ea58`, each fresh parser
process consumed 1,073,741,824 bytes of NDJSON and verified 4,227,072 documents
and 46,497,792 events through EOF. A repeating 32 KiB block supplied 128 short
records and one 16 KiB record per block. The generator and the parser shared a
process. The input was generated in memory.

| Read buffer (bytes) | Peak RSS (KiB) | Limit (bytes) | Result |
| ---: | ---: | ---: | :--- |
| 1 | 1,992 | 8,388,612 | Pass |
| 7 | 1,936 | 8,388,636 | Pass |
| 64 | 2,056 | 8,388,864 | Pass |
| 8,192 | 1,940 | 8,421,376 | Pass |

The limit is 8 MiB plus four times the read buffer. RSS is the Linux kernel's
process high-water mark (`VmHWM`), read after EOF and before process exit.
The test ran on `arbi-pc` under WSL2 Linux, AMD Ryzen 9 9950X3D2, Rust 1.100.0,
with the `read` feature and a release build. The cgroup allowed 25 ms per 100 ms
on one logical CPU. The load guard delayed the final run until three consecutive readings were below
the thresholds.

## Small-document allocations and latency on Ryzen 9

All four inputs come from `benches/support/small_inputs.rs`. The allocation
test runs in its own single-threaded executable. A boxed value verifies that
the counter sees heap activity. Each fixture starts with a fresh 0.2 parser,
parses once to warm it, then parses the same input another 1,000 times.

| Input | Bytes | Cold allocations | Warm allocations | Warm reallocations | Warm deallocations |
| --- | ---: | ---: | ---: | ---: | ---: |
| `tiny_ok` | 11 | 2 | 0 | 0 | 0 |
| `ticker` | 169 | 2 | 0 | 0 | 0 |
| `escaped` | 73 | 2 | 0 | 0 | 0 |
| `nested` | 198 | 2 | 0 | 0 | 0 |

Warm counts are totals over all 1,000 calls per row. Larger or deeper inputs
can grow the retained buffers again. The dev dependency `stats_alloc 0.1.10`
provides the counting allocator for this test. A separate Criterion executable
uses the normal allocator for timing. Its `rs-json-010` dev dependency pins
the published `rs-json 0.1.0`, so both versions run in the same executable.
Each one-shot iteration parses the document and drops the resulting DOM.
The reuse iteration returns a document view and retains the parser's buffers.

The benchmark harness at `9f1f356` uses the 0.2 parser from `20bcd689`.
Criterion 0.8.2 ran on the same Ryzen 9 and Rust 1.100.0, with default crate
features, one Cargo job, one codegen unit, and LTO and debug symbols disabled.
The benchmark used logical CPU 21, nice 15, and a cgroup quota of 25 ms per
100 ms. These are wall-clock latencies under that quota. The load guard halted
work at 35% Windows or WSL CPU utilization, or a WSL one-minute load of 7.7
on 22 logical CPUs. An interrupted timing run restarted at warmup after three
consecutive readings below all thresholds.

Timing cells give **ns/document (MiB/s)** from Criterion’s regression slope.
Each measurement used a one-second warmup, 20 samples over eight seconds, and
10,000 bootstrap resamples. Speedup divides the 0.1.0 time by the 0.2 time.
Values above 1 favor 0.2.

### Default x86-64 target

| Input | 0.1.0 one-shot | 0.2 one-shot | 0.2 reuse | One-shot speedup | Reuse speedup |
| --- | ---: | ---: | ---: | ---: | ---: |
| `tiny_ok` | 123.6 (84.9) | 211.2 (49.7) | 149.9 (70.0) | 0.59× | 0.82× |
| `ticker` | 813.9 (198.0) | 1506.8 (107.0) | 1248.1 (129.1) | 0.54× | 0.65× |
| `escaped` | 224.3 (310.3) | 497.7 (139.9) | 342.2 (203.4) | 0.45× | 0.66× |
| `nested` | 1687.0 (111.9) | 2373.1 (79.6) | 1985.5 (95.1) | 0.71× | 0.85× |

### Native Ryzen target

| Input | 0.1.0 one-shot | 0.2 one-shot | 0.2 reuse | One-shot speedup | Reuse speedup |
| --- | ---: | ---: | ---: | ---: | ---: |
| `tiny_ok` | 124.2 (84.4) | 192.3 (54.5) | 143.1 (73.3) | 0.65× | 0.87× |
| `ticker` | 784.8 (205.4) | 1244.6 (129.5) | 1069.2 (150.7) | 0.63× | 0.73× |
| `escaped` | 220.5 (315.7) | 459.9 (151.4) | 324.2 (214.7) | 0.48× | 0.68× |
| `nested` | 1596.1 (118.3) | 2178.8 (86.7) | 1828.3 (103.3) | 0.73× | 0.87× |

The allocation target passed on all four fixtures. The G1 goal of 2× the
0.1.0 parsing speed remains open. Even with a warmed parser, 0.2 took
18–53% more time with the default target and 15–47% more time with native
code generation. These measurements cover the slice DOM entry points.

To reproduce one timing in a shell with the same CPU quota, affinity and load
guard, substitute any fixture and lane from the tables. For the native pass,
replace `unset RUSTFLAGS` with `export RUSTFLAGS="-C target-cpu=native"`.

```sh
export CARGO_BUILD_JOBS=1 CARGO_PROFILE_BENCH_LTO=false CARGO_PROFILE_BENCH_DEBUG=0
unset RUSTFLAGS
cargo test --locked --profile bench --test alloc_count
cargo bench --locked --bench parse_small -- 'small/ticker/rs_json_010::parse' \
  --exact --warm-up-time 1 --measurement-time 8 --sample-size 20 --nresamples 10000 --noplot
```

The 0.2 lane IDs are `rs_json::parse` and `rs_json::Parser(reuse)`.

## Compact DOM tuning on arbi-pc

On the merged PR #3 parser (`06cf6da`), a Linux CPU-time sampler collected
530 instruction-pointer samples during eight million reused parses of the four
fixtures. The slice lexer's `next_token` accounted for 45.1%, the grammar's
`step` for 28.9%, and the DOM tape builder for 17.9%.

The tuned build (`63e485f`) writes compact UTF-8 documents up to 4 KiB
directly to the tape. Open containers temporarily store their parent index in
the tape. Short owned documents can hold packed nodes inline. Other inputs use
the general grammar, which also supplies the error kind and byte position when
the compact path declines an input.

A reusable parser can retain token boundaries for inputs up to 512 bytes.
After two successful parses of the same length it can validate a subsequent
input against byte ranges for syntax, string bodies, number digits and Unicode
escapes. It checks every byte on a match and returns a view of the current
input. A mismatch takes the regular parsing path. The one-shot API builds each
document independently.

All timings below come from Rust 1.100.0 on the Ryzen 9 9950X3D2 under
WSL2. The benchmark used logical CPU 21, nice 15, a 25 ms quota per 100 ms,
a 1 GiB cgroup memory limit and one Cargo build job. The guard pauses work
at 35% host or guest CPU or a guest one-minute load of 7.7 and requires
three low samples before starting or resuming. Criterion used the same
one-second warmup, 20 samples over eight seconds and 10,000 bootstrap resamples
as the preceding run. Timings are wall-clock regression slopes under the quota.
Speedup divides the freshly measured 0.1.0 time by the 0.2 time in the same build.

### Default x86-64

| Input | 0.1.0 ns/doc | 0.2 one-shot ns/doc | One-shot speedup | 0.2 reuse ns/doc | Reuse speedup |
| --- | ---: | ---: | ---: | ---: | ---: |
| `tiny_ok` | 120.0 | 48.8 | 2.46× | 28.4 | 4.22× |
| `ticker` | 770.6 | 285.0 | 2.70× | 45.8 | 16.82× |
| `escaped` | 226.5 | 130.6 | 1.74× | 34.5 | 6.57× |
| `nested` | 1538.7 | 460.3 | 3.34× | 45.2 | 34.02× |

### Native Ryzen

| Input | 0.1.0 ns/doc | 0.2 one-shot ns/doc | One-shot speedup | 0.2 reuse ns/doc | Reuse speedup |
| --- | ---: | ---: | ---: | ---: | ---: |
| `tiny_ok` | 143.5 | 48.5 | 2.96× | 29.8 | 4.82× |
| `ticker` | 760.3 | 288.9 | 2.63× | 54.3 | 14.01× |
| `escaped` | 212.4 | 130.3 | 1.63× | 33.0 | 6.43× |
| `nested` | 1520.0 | 500.8 | 3.03× | 55.0 | 27.62× |

### Different inputs on successive calls

The first four controls alternate distinct input buffers of the same length.
They change keys, numbers, strings or escaped Unicode characters while keeping
token boundaries stable. The final control cycles all four original documents
through one parser, changing the layout on every call. Each value below is per
document (two or four documents per measured iteration).

| Workload | 0.1.0 default ns | 0.2 reuse default ns | Speedup | 0.1.0 native ns | 0.2 reuse native ns | Speedup |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `tiny_ok` | 121.8 | 33.7 | 3.62× | 120.7 | 36.3 | 3.32× |
| `ticker` | 787.2 | 45.3 | 17.38× | 854.9 | 54.3 | 15.75× |
| `escaped` | 218.6 | 35.3 | 6.19× | 214.5 | 42.3 | 5.07× |
| `nested` | 1540.9 | 46.2 | 33.38× | 1524.4 | 55.2 | 27.59× |
| `mixed_shapes` | 664.8 | 226.5 | 2.93× | 685.0 | 230.1 | 2.98× |

### Allocation and speed targets

A fresh reusable parser made three allocation calls on its first parse of each
fixture. Each subsequent 1,000-parse run had zero allocations, reallocations and
deallocations. The mixed test also recorded zero heap operations over 8,000 calls
after warming the parser on the four input sizes.

The reused parser passed the 2× target on every fixed fixture in both builds and
on each alternating-payload control. With a new owned document on each call,
three fixtures passed in each build. At that revision, one-shot escaped strings
measured 1.74× default and 1.63× native.

The existing `small` benchmark IDs are unchanged. To reproduce the controls,
run the following command under the same CPU guard. Repeat the run with
`RUSTFLAGS="-C target-cpu=native"`.

```sh
CARGO_BUILD_JOBS=1 CARGO_PROFILE_BENCH_LTO=false CARGO_PROFILE_BENCH_DEBUG=0 \
  cargo bench --locked --bench parse_small -- 'rotating/' \
  --warm-up-time 1 --measurement-time 8 --sample-size 20 --nresamples 10000 --noplot
```


The same host passed formatting, strict all-target Clippy, all-feature and
no-default-feature tests, bench compilation, and all 318 pinned JSONTestSuite
cases. Differential tests checked the optimized tapes against the byte parser,
including changed fields, depth-limit changes, malformed inputs and UTF-16
surrogate boundaries.

## One-shot escaped strings on arbi-pc

On 64-bit targets with the `simd` feature, commit `dace0a1` indexes the string
boundaries in a compact object with one or two string values using a 64-bit mask. It
validates escape sequences and UTF-16 surrogate pairs before returning the inline
tape. Every one-shot call starts from its input. The reused parser follows the same
implementation as in `b154383`.

All four fixtures ran again on the Ryzen 9 with Rust 1.100.0. The published 0.1.0
baseline ran in the candidate executable. Separate executables built from the previous
PR head (`b154383`) and the candidate used the same locked dependencies, features,
optimization settings and target flags. Adjacent measurements compared the two 0.2
executables, alternating their order across fixtures and builds.

| Build | Input | 0.1.0 ns | Prior one-shot ns | New one-shot ns | One-shot vs 0.1.0 | Prior reuse ns | New reuse ns | Reuse vs 0.1.0 |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| default | `tiny_ok` | 123.9 | 47.8 | 47.4 | 2.61× | 28.4 | 28.5 | 4.35× |
| default | `ticker` | 770.3 | 280.4 | 285.8 | 2.69× | 45.3 | 45.2 | 17.05× |
| default | `escaped` | 212.8 | 128.6 | 66.6 | 3.19× | 34.2 | 34.4 | 6.19× |
| default | `nested` | 1517.6 | 453.8 | 451.1 | 3.36× | 44.8 | 44.9 | 33.83× |
| native | `tiny_ok` | 121.3 | 47.8 | 47.7 | 2.54× | 30.0 | 29.9 | 4.06× |
| native | `ticker` | 761.8 | 289.3 | 290.7 | 2.62× | 53.9 | 54.1 | 14.09× |
| native | `escaped` | 215.4 | 132.7 | 68.7 | 3.14× | 33.2 | 33.4 | 6.45× |
| native | `nested` | 1495.1 | 493.8 | 493.6 | 3.03× | 54.6 | 55.4 | 26.99× |

The one-shot escaped fixture took 48% less time than the previous PR head in each
build. For the other three one-shot fixtures and every reused-parser fixture, point
estimates differed by less than 2% and the paired runs had overlapping 95% slope
intervals.

Each entry is a Criterion slope in wall-clock nanoseconds per document. Each lane used
a one-second warmup, 20 samples over eight seconds and 10,000 bootstrap resamples. The
guard limited builds and benchmarks to 25% of one logical core (CPU 21), nice 15,
one build job and 1 GiB of memory. It restarts an interrupted timing from warmup
after three consecutive readings with Windows and WSL CPU utilization both below 35%
and guest one-minute load below 7.7. Default and `-C target-cpu=native` used the
same guard and benchmark inputs.

### Changing inputs and allocations

The first four controls alternate different payloads with the same token boundaries.
The last rotates through all four document shapes. The table gives reused-parser
speedup over published 0.1.0.

| Control | Default | Native |
| --- | ---: | ---: |
| `tiny_ok` | 3.55× | 3.39× |
| `ticker` | 17.39× | 14.29× |
| `escaped` | 6.15× | 6.26× |
| `nested` | 32.64× | 27.52× |
| `mixed_shapes` | 2.89× | 2.93× |

In each build, all four fixtures recorded zero allocations, reallocations or
deallocations over 1,000 parses per fixture after warmup. The 8,000-parse mixed
sequence also recorded zero heap operations.

To repeat the fixed-input lanes, use the `small/<fixture>/<lane>` exact-filter command
above. Compare a checkout of `b154383` with `dace0a1` using separate Criterion output
directories and repeat the rotating controls with the same build settings.

## Corpus benchmarks on arbi-pc

These measurements use the parser at `ebc613f`, the merge of PR #4 into
`staging/0.2-skeleton`, with the corpus harness from `7dab6eb`. Each timed iteration
consumes one complete in-memory document.
Time is the Criterion mean in nanoseconds per document. Throughput is decimal MB/s,
calculated from the exact file length and the unrounded time.

| Document | Operation | Default ns/doc | Default MB/s | Native ns/doc | Native MB/s |
| --- | --- | ---: | ---: | ---: | ---: |
| `twitter.json` | rs-json 0.2 one-shot | 1,705,152 | 370.4 | 1,886,341 | 334.8 |
|  | rs-json 0.2 reused | 1,909,103 | 330.8 | 1,712,278 | 368.8 |
|  | rs-json at `ae68abb` | 2,215,032 | 285.1 | 2,490,542 | 253.6 |
|  | serde_json DOM | 4,107,787 | 153.7 | 4,059,626 | 155.6 |
|  | sonic-rs DOM | 1,052,085 | 600.2 | 689,597 | 915.8 |
|  | simd-json borrowed DOM | 3,151,419 | 200.4 | 3,166,924 | 199.4 |
|  | bufjson validation | 1,842,535 | 342.7 | 1,817,112 | 347.5 |
| `citm_catalog.json` | rs-json 0.2 one-shot | 5,157,679 | 334.9 | 4,623,733 | 373.6 |
|  | rs-json 0.2 reused | 4,733,954 | 364.9 | 4,493,282 | 384.4 |
|  | rs-json at `ae68abb` | 4,544,173 | 380.1 | 4,064,024 | 425.0 |
|  | serde_json DOM | 8,411,578 | 205.3 | 7,172,111 | 240.8 |
|  | sonic-rs DOM | 2,899,158 | 595.8 | 1,522,206 | 1134.7 |
|  | simd-json borrowed DOM | 9,366,849 | 184.4 | 9,297,035 | 185.8 |
|  | bufjson validation | 4,699,375 | 367.5 | 4,618,557 | 374.0 |
| `canada.json` | rs-json 0.2 one-shot | 9,487,787 | 237.3 | 9,400,328 | 239.5 |
|  | rs-json 0.2 reused | 9,713,786 | 231.7 | 9,664,292 | 232.9 |
|  | rs-json at `ae68abb` | 17,015,817 | 132.3 | 17,508,476 | 128.6 |
|  | serde_json DOM | 26,426,695 | 85.2 | 26,810,720 | 84.0 |
|  | sonic-rs DOM | 5,822,630 | 386.6 | 5,541,689 | 406.2 |
|  | simd-json borrowed DOM | 23,277,129 | 96.7 | 21,864,355 | 103.0 |
|  | bufjson validation | 6,361,999 | 353.8 | 6,102,862 | 368.9 |
| `gen_mixed_16mb.json` | rs-json 0.2 one-shot | 136,631,013 | 123.0 | 134,711,638 | 124.7 |
|  | rs-json 0.2 reused | 111,140,006 | 151.2 | 107,933,490 | 155.7 |
|  | rs-json at `ae68abb` | 176,412,068 | 95.3 | 176,668,851 | 95.1 |
|  | serde_json DOM | 273,451,544 | 61.5 | 263,125,433 | 63.9 |
|  | sonic-rs DOM | 88,852,544 | 189.1 | 85,038,562 | 197.6 |
|  | simd-json borrowed DOM | 253,811,782 | 66.2 | 250,521,475 | 67.1 |
|  | bufjson validation | 88,954,721 | 188.9 | 84,830,864 | 198.1 |

### Comparison with the historical parser and sonic-rs

The old-parser speedup divides historical one-shot time by current one-shot time.
The sonic-rs ratio divides current one-shot time by sonic-rs DOM time. The S4 DOM
target is at most 1.30× sonic-rs latency on Twitter and CITM. The one-shot point estimates
exceeded that target on both fixtures in both builds.

| Document | Speedup vs old, default | Speedup vs old, native | Time / sonic-rs, default | Time / sonic-rs, native |
| --- | ---: | ---: | ---: | ---: |
| `twitter.json` | 1.30× | 1.32× | 1.62× | 2.74× |
| `citm_catalog.json` | 0.88× | 0.88× | 1.78× | 3.04× |
| `canada.json` | 1.79× | 1.86× | 1.63× | 1.70× |
| `gen_mixed_16mb.json` | 1.29× | 1.31× | 1.54× | 1.58× |

One-shot 0.2 had a lower mean time than the historical parser on three documents in
both builds. CITM was the exception at 0.88×. Under this CPU quota its individual
95% bootstrap intervals overlapped. In milliseconds, the default intervals were
4.32–6.38 for 0.2 and 3.85–5.54 for the historical parser. The native intervals
were 4.10–5.20 and 3.70–4.37, respectively.

The one-shot DOM lanes include construction and destruction of an owned or borrowed DOM.
Reusing `rs_json::Parser` retains its tape between parses. The simd-json lane
prepares one fresh mutable input buffer per iteration outside the timer and drops
the parsed DOM inside it. The bufjson lane validates through EOF without building
a DOM. File I/O is outside all timings.

### Inputs and reproduction

| File | Bytes | SHA-256 |
| --- | ---: | --- |
| `twitter.json` | 631,514 | `a08b769f32b95f426cbc3abafcec65c1a19d3eb544d4ddf320eae142c99efc5d` |
| `citm_catalog.json` | 1,727,204 | `a73e7a883f6ea8de113dff59702975e60119b4b58d451d518a929f31c92e2059` |
| `canada.json` | 2,251,051 | `f83b3b354030d5dd58740c68ac4fecef64cb730a0d12a90362a7f23077f50d78` |
| `gen_mixed_16mb.json` | 16,804,466 | `54245962ae96aecb9de6f3360d58b595178e6173fa3b55f1e6174a05ce2384bd` |

The three public documents come from [nativejson-benchmark at `478d572`](https://github.com/miloyip/nativejson-benchmark/tree/478d5727c2a4048e835a29c65adecc7d795360d5/data).
The mixed array comes from `examples/gen_corpus.rs` at the measured merge.
The historical lane compiled the Git sources of `rust-misc/rs-json` at
`ae68abb54c655a8fcc009123fbc31821b82435e4` through a temporary path dependency.
All 32 files in its `src` tree (`c6cf481d22b3ab50ef6835605865ed4b1a4e0375`)
matched the public 0.1.0 package byte for byte. The checked-in harness uses that
package. The other versions and checksums matched the staging lockfile, including
Criterion 0.8.2, serde_json 1.0.154, sonic-rs 0.5.12, simd-json 0.19.0 and
bufjson 1.3.0.

The host was an AMD Ryzen 9 9950X3D2 under WSL2 Linux `6.18.58.1-microsoft-standard-WSL2`, with Rust
1.100.0. Default used no `RUSTFLAGS`. Native used `-C target-cpu=native`.
Both used CPU 21, nice 15, a quota of 25 ms per 100 ms, a 1 GiB memory limit
and one build job. The guard required three consecutive readings below 35% Windows
and WSL CPU and below 7.7 guest one-minute load. An interrupted lane restarts
from warmup. These wall-clock rates include scheduling under the quota.

For comparable wall-clock timings, run from the repository root on the benchmark
host inside the CPU quota and load guard described above.

```sh
set -eu
export CARGO_BUILD_JOBS=1 CARGO_PROFILE_BENCH_LTO=false CARGO_PROFILE_BENCH_DEBUG=0
mkdir -p data
base=https://raw.githubusercontent.com/miloyip/nativejson-benchmark/478d5727c2a4048e835a29c65adecc7d795360d5/data
for f in twitter.json citm_catalog.json canada.json; do
    curl -fsSL "$base/$f" -o "data/$f"
done
cargo run --locked --profile bench --example gen_corpus -- data/gen_mixed_16mb.json mixed 16
for mode in default native; do
    if [ "$mode" = native ]; then export RUSTFLAGS='-C target-cpu=native'; else unset RUSTFLAGS; fi
    export CRITERION_HOME="$PWD/target/corpus-$mode"
    for f in twitter.json citm_catalog.json canada.json gen_mixed_16mb.json; do
        for lane in rs_json::parse 'rs_json::Parser(reuse)' rs_json_010::parse serde_json::Value sonic_rs::Value simd_json::borrowed bufjson::validate; do
            cargo bench --locked --bench parse_corpus -- "corpus/$f/$lane" --exact \
                --warm-up-time 1 --measurement-time 8 --nresamples 10000 --noplot
        done
    done
done
```

A smoke run parsed or validated every fixture with every lane before timing. Each
build produced all 28 expected measurements, with 30 samples per lane.

On the benchmark host, the checked-in dependency configuration passed
`cargo fmt --all --check`, strict Clippy across all features and targets,
tests with all features and with default features disabled, and
`cargo check --locked --benches --all-features`.
