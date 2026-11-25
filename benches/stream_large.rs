//! Large-file paths: read-to-Vec vs mmap vs buffered streaming, and the
//! read-buffer sweep (8/16/64/256 KiB). Report MB/s *and* peak RSS
//! (`/usr/bin/time -v` or the counting allocator in `tests/alloc_count.rs`).
//! Generate inputs with `scripts/gen-large.sh` (256 MB and 1 GB arrays + NDJSON).

use criterion::{Criterion, Throughput, criterion_group, criterion_main};

const FILES: &[&str] = &["gen_array_256mb.json", "gen_lines_256mb.ndjson"];
const BUF_SIZES: &[usize] = &[8 << 10, 16 << 10, 64 << 10, 256 << 10];

#[cfg(feature = "read")]
fn reader_buffers(c: &mut Criterion) {
    use rs_json::{
        Limits,
        config::BufConfig,
        stream::{Event, PullParser},
    };
    // Large enough to cross even the 8192-byte buffer; entirely in memory.
    let record = r#"{"k":"mémoire \u00e9","v":[-12.3e+4,true,null]}"#;
    let document = format!("[{}]", vec![record; 256].join(","));
    let mut group = c.benchmark_group("reader_buffers");
    group.throughput(Throughput::Bytes(document.len() as u64));
    for size in [1, 7, 64, 8192] {
        group.bench_function(size.to_string(), |b| {
            b.iter(|| {
                let mut p = PullParser::from_reader(
                    std::hint::black_box(document.as_bytes()),
                    BufConfig {
                        read_buf: size,
                        ..BufConfig::default()
                    },
                    Limits::default(),
                );
                while p.next_event().unwrap() != Event::Eof {}
            });
        });
    }
    group.finish();
}

fn bench(c: &mut Criterion) {
    #[cfg(feature = "read")]
    reader_buffers(c);
    for file in FILES {
        let path = format!("{}/data/{file}", env!("CARGO_MANIFEST_DIR"));
        let Ok(meta) = std::fs::metadata(&path) else {
            eprintln!("skip {file}: run scripts/gen-large.sh");
            continue;
        };
        let mut g = c.benchmark_group(format!("large/{file}"));
        g.throughput(Throughput::Bytes(meta.len()));
        g.sample_size(10);
        for &buf in BUF_SIZES {
            g.bench_function(format!("bufjson::ReadAnalyzer/{}k", buf >> 10), |b| {
                b.iter(|| {
                    use bufjson::{lexical::read::ReadAnalyzer, syntax::Parser};
                    let f = std::fs::File::open(&path).unwrap();
                    let mut p = Parser::new(ReadAnalyzer::with_buf_size(f, buf));
                    while !p.next().is_terminal() {}
                });
            });
            // g.bench_function(format!("rs_json::PullParser/{}k", buf >> 10), ...);
        }
        // g.bench_function("rs_json::parse_file(mmap)", ...);
        // g.bench_function("rs_json::parse_file(read)", ...);
        g.finish();
    }
}

criterion_group!(benches, bench);
criterion_main!(benches);
