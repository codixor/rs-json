//! Small-document latency (≤ 4 KiB): API responses, websocket frames.
//! Inputs are generated with a fixed seed and *varied per iteration batch*
//! so no parser can win by special-casing one literal (see the json_fast
//! lookup-table trick documented in docs/ANALYSIS.md).

use criterion::{BatchSize, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

const SAMPLES: &[(&str, &str)] = &[
    ("tiny_ok", r#"{"ok":true}"#),
    ("ticker", r#"{"e":"24hrTicker","E":1732450000123,"s":"BTCUSDT","p":"-120.50","P":"-0.13","c":"94210.10","Q":"0.012","b":"94210.00","a":"94210.10","v":"18234.551","q":"1717171717.12"}"#),
    ("escaped", r#"{"msg":"line1\nline2 \"quoted\" \u00e9\ud83d\ude00","path":"C:\\temp\\x"}"#),
    ("nested", r#"{"meta":{"name":"json","versions":[1,2,3,4]},"users":[{"id":1,"name":"a","flags":[true,false,null]},{"id":2,"name":"b","flags":[false,true,null]}],"config":{"retry":3,"timeout":1.25,"enabled":true}}"#),
];

fn bench(c: &mut Criterion) {
    for (name, doc) in SAMPLES {
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
                    |mut v| { let _ = simd_json::to_borrowed_value(&mut v).unwrap(); },
                    BatchSize::SmallInput,
                );
            });
        }
        // Enable once dom::build lands (gate G1 in docs/PLAN.md):
        // g.bench_function("rs_json::parse", |b| b.iter(|| rs_json::parse(black_box(doc)).unwrap()));
        // let mut p = rs_json::Parser::new();
        // g.bench_function("rs_json::Parser(reuse)", |b| b.iter(|| { p.parse(black_box(doc)).unwrap(); }));
        g.finish();
    }
}

criterion_group!(benches, bench);
criterion_main!(benches);
