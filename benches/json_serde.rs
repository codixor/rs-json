use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::hint::black_box;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Profile {
    id: u64,
    name: String,
    active: bool,
    roles: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct BenchDocument {
    request_id: String,
    profile: Profile,
    tags: Vec<String>,
    metadata: BTreeMap<String, String>,
    events: Vec<Profile>,
}

fn sample_document() -> BenchDocument {
    let profile = Profile {
        id: 7,
        name: String::from("alice"),
        active: true,
        roles: vec![String::from("admin"), String::from("writer")],
    };

    let mut metadata = BTreeMap::new();
    metadata.insert(String::from("region"), String::from("me"));
    metadata.insert(String::from("env"), String::from("bench"));
    metadata.insert(String::from("tenant"), String::from("workspace"));

    let events = (0..24)
        .map(|index| Profile {
            id: index,
            name: format!("user-{index}"),
            active: index % 2 == 0,
            roles: vec![String::from("reader"), format!("cohort-{}", index % 4)],
        })
        .collect();

    BenchDocument {
        request_id: String::from("req_01HQYVQ9A0CACHE"),
        profile,
        tags: vec![
            String::from("cache"),
            String::from("json"),
            String::from("serializer"),
            String::from("benchmark"),
        ],
        metadata,
        events,
    }
}

fn bench_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("json/serde_encode");
    let document = sample_document();

    group.bench_function(BenchmarkId::new("json", "workspace_facade"), |b| {
        b.iter(|| {
            let encoded = rs_json::to_vec(black_box(&document)).unwrap();
            black_box(encoded);
        });
    });

    group.bench_function(BenchmarkId::new("json", "serde_json_direct"), |b| {
        b.iter(|| {
            let encoded = serde_json::to_vec(black_box(&document)).unwrap();
            black_box(encoded);
        });
    });

    group.finish();
}

fn bench_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("json/serde_decode");
    let encoded = rs_json::to_vec(&sample_document()).unwrap();

    group.bench_function(BenchmarkId::new("json", "workspace_facade"), |b| {
        b.iter(|| {
            let decoded: BenchDocument = rs_json::from_slice(black_box(&encoded)).unwrap();
            black_box(decoded);
        });
    });

    group.bench_function(BenchmarkId::new("json", "serde_json_direct"), |b| {
        b.iter(|| {
            let decoded: BenchDocument = serde_json::from_slice(black_box(&encoded)).unwrap();
            black_box(decoded);
        });
    });

    group.finish();
}

criterion_group!(benches, bench_encode, bench_decode);
criterion_main!(benches);
