//! In-memory DOM parsing and syntax validation on three nativejson-benchmark
//! documents and a deterministic mixed array. Each iteration processes a full
//! document. Generate missing fixtures as described in docs/BENCH.md.

use bufjson::{
    lexical::{Token, fixed::FixedAnalyzer},
    syntax::Parser,
};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

const FILES: &[&str] = &[
    "twitter.json",
    "citm_catalog.json",
    "canada.json",
    "gen_mixed_16mb.json",
];

fn bench(c: &mut Criterion) {
    for file in FILES {
        let path = format!("{}/data/{file}", env!("CARGO_MANIFEST_DIR"));
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                eprintln!("skip {file}: see docs/BENCH.md for corpus setup");
                continue;
            }
            Err(err) => panic!("cannot read {path}: {err}"),
        };
        let mut g = c.benchmark_group(format!("corpus/{file}"));
        g.throughput(Throughput::Bytes(text.len() as u64));
        g.sample_size(30);
        // The generated 16 MiB fixture includes a final record beyond the default cap.
        let defaults = rs_json::Limits::default();
        let limits = rs_json::Limits {
            max_document_bytes: defaults.max_document_bytes.max(text.len() as u64),
            ..defaults
        };
        g.bench_function("rs_json::parse", |b| {
            if limits == defaults {
                b.iter(|| rs_json::parse(black_box(&text)).unwrap());
            } else {
                b.iter(|| rs_json::parse_with_limits(black_box(&text), limits).unwrap());
            }
        });
        let mut parser = rs_json::Parser::new();
        parser.limits = limits;
        g.bench_function("rs_json::Parser(reuse)", |b| {
            b.iter(|| {
                black_box(parser.parse(black_box(&text)).unwrap());
            });
        });
        g.bench_function("rs_json_010::parse", |b| {
            b.iter(|| rs_json_010::parse(black_box(&text)).unwrap());
        });
        g.bench_function("serde_json::Value", |b| {
            b.iter(|| serde_json::from_str::<serde_json::Value>(black_box(&text)).unwrap());
        });
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
        {
            g.bench_function("sonic_rs::Value", |b| {
                b.iter(|| sonic_rs::from_str::<sonic_rs::Value>(black_box(&text)).unwrap());
            });
            g.bench_function("simd_json::borrowed", |b| {
                // Parsing mutates the input. Prepare one fresh buffer outside
                // each timing and include destruction of the DOM inside it.
                b.iter_batched_ref(
                    || text.as_bytes().to_vec(),
                    |bytes| {
                        let value = simd_json::to_borrowed_value(black_box(bytes)).unwrap();
                        drop(black_box(value));
                    },
                    criterion::BatchSize::PerIteration,
                );
            });
        }
        g.bench_function("bufjson::validate", |b| {
            b.iter(|| {
                let mut p = Parser::new(FixedAnalyzer::new(black_box(text.as_bytes())));
                assert_eq!(p.next_end(), Token::Eof, "{file}: {}", p.err());
            });
        });
        g.finish();
    }
}

criterion_group!(benches, bench);
criterion_main!(benches);
