//! Small-document latency (≤ 4 KiB): API responses, websocket frames.
//! Each case measures one fixed input so timings are comparable across parsers.

use criterion::{BatchSize, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

#[path = "support/small_inputs.rs"]
mod small_inputs;

fn bench(c: &mut Criterion) {
    for (name, doc) in small_inputs::SAMPLES {
        let mut g = c.benchmark_group(format!("small/{name}"));
        g.throughput(Throughput::Bytes(doc.len() as u64));
        g.bench_function("serde_json::Value", |b| {
            b.iter(|| serde_json::from_str::<serde_json::Value>(black_box(doc)).unwrap());
        });
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
        {
            g.bench_function("sonic_rs::Value", |b| {
                b.iter(|| sonic_rs::from_str::<sonic_rs::Value>(black_box(doc)).unwrap());
            });
            g.bench_function("simd_json::borrowed", |b| {
                b.iter_batched(
                    || doc.as_bytes().to_vec(),
                    |mut v| {
                        let _ = simd_json::to_borrowed_value(&mut v).unwrap();
                    },
                    BatchSize::SmallInput,
                );
            });
        }
        g.bench_function("rs_json_010::parse", |b| {
            b.iter(|| rs_json_010::parse(black_box(doc)).unwrap());
        });
        g.bench_function("rs_json::parse", |b| {
            b.iter(|| rs_json::parse(black_box(doc)).unwrap());
        });
        let mut parser = rs_json::Parser::new();
        g.bench_function("rs_json::Parser(reuse)", |b| {
            b.iter(|| {
                black_box(parser.parse(black_box(doc)).unwrap());
            });
        });
        g.finish();
    }
}

// Process every document in a cycle per iteration. Each returned document
// borrows its own input, and setup stays outside the timed loop.
fn cycle(c: &mut Criterion, name: &str, docs: &[&str]) {
    let mut group = c.benchmark_group(format!("rotating/{name}"));
    group.throughput(Throughput::Elements(docs.len() as u64));
    group.bench_function("rs_json_010::parse", |b| {
        b.iter(|| {
            for &doc in black_box(docs) {
                black_box(rs_json_010::parse(black_box(doc)).unwrap());
            }
        });
    });
    let mut parser = rs_json::Parser::new();
    group.bench_function("rs_json::Parser(reuse)", |b| {
        b.iter(|| {
            for &doc in black_box(docs) {
                black_box(parser.parse(black_box(doc)).unwrap());
            }
        });
    });
    group.finish();
}

fn rotating(c: &mut Criterion) {
    for &(name, original) in small_inputs::SAMPLES {
        let other = small_inputs::alternate(name, original);
        cycle(c, name, &[original, &other]);
    }
    let mixed: Vec<_> = small_inputs::SAMPLES.iter().map(|&(_, s)| s).collect();
    cycle(c, "mixed_shapes", &mixed);
}

criterion_group!(benches, bench, rotating);
criterion_main!(benches);
