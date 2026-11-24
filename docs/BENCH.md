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

The S2 performance gate still needs a comparison with 0.1 on the same inputs
and a steady-state allocation count. The larger corpus, reader benchmarks and
native-CPU measurements belong to the remaining roadmap.

## Reader refill sizes

The reader consumes the same 12,545-byte JSON array of 256 records at each buffer
size. Each iteration creates a reader and consumes every event through EOF.
The input is memory-backed. These times measure tokenization and refill
handling. File I/O and peak resident memory require separate measurements.

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
