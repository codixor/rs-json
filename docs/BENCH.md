# 0.2 slice parser measurements

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
three fixtures passed in each build. One-shot escaped strings remain below
target at 1.74× default and 1.63× native.

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
