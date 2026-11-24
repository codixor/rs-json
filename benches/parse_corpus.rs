//! Medium/large in-memory documents from the nativejson-benchmark corpus
//! (twitter.json 617 KB string/unicode heavy, `citm_catalog.json` 1.7 MB
//! structure heavy, canada.json 2.2 MB float heavy) plus generated files.
//! Run `scripts/fetch-corpus.sh` first; missing files are skipped.

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
        let Ok(text) = std::fs::read_to_string(&path) else {
            eprintln!("skip {file}: run scripts/fetch-corpus.sh");
            continue;
        };
        let mut g = c.benchmark_group(format!("corpus/{file}"));
        g.throughput(Throughput::Bytes(text.len() as u64));
        g.sample_size(30);
        g.bench_function("serde_json::Value", |b| {
            b.iter(|| serde_json::from_str::<serde_json::Value>(black_box(&text)).unwrap());
        });
        g.bench_function("bufjson::validate", |b| {
            b.iter(|| {
                use bufjson::{lexical::fixed::FixedAnalyzer, syntax::Parser};
                let mut p = Parser::new(FixedAnalyzer::new(black_box(text.as_bytes())));
                while !p.next().is_terminal() {}
            });
        });
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
        g.bench_function("sonic_rs::Value", |b| {
            b.iter(|| sonic_rs::from_str::<sonic_rs::Value>(black_box(&text)).unwrap());
        });
        // g.bench_function("rs_json::parse", |b| b.iter(|| rs_json::parse(black_box(&text)).unwrap()));
        g.finish();
    }
}

criterion_group!(benches, bench);
criterion_main!(benches);
