use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};

const SMALL_OBJECT: &str = r#"{"name":"json","ready":true,"count":3,"tags":["fast","safe"]}"#;
const NESTED_OBJECT: &str = r#"{
  "meta":{"name":"json","versions":[1,2,3,4]},
  "users":[
    {"id":1,"name":"a","flags":[true,false,null]},
    {"id":2,"name":"b","flags":[false,true,null]}
  ],
  "config":{"retry":3,"timeout":1.25,"enabled":true}
}"#;
const INVALID_DOCUMENT: &str = r#"{"meta":{"name":"json",}}"#;
const STRING_HEAVY_OBJECT: &str = r#"{
  "title":"json benchmark",
  "body":"the quick brown fox jumps over the lazy dog while json parsing stays zero-copy",
  "meta":{
    "author":"workspace",
    "summary":"a longer string payload that exercises string scanning and escape-safe parsing"
  },
  "items":[
    "alpha beta gamma delta epsilon",
    "one two three four five six seven eight",
    "symbols !@#$%^&*()[]{}<> stay plain text here"
  ]
}"#;
const NUMBER_HEAVY_OBJECT: &str = r#"{
  "ints":[1,2,3,4,5,6,7,8,9,10,11,12],
  "floats":[0.125,1.25,2.5,3.75,4.125,5.5,6.875,7.25],
  "exp":[1e3,2e4,3e5,4e6,5e7,6e8]
}"#;

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
fn bench_simd_json(document: &str) {
    let mut bytes = document.as_bytes().to_vec();
    let _ = simd_json::to_owned_value(&mut bytes).unwrap();
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
fn bench_simd_json(_document: &str) {}

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
fn bench_sonic_rs(document: &str) {
    let _ = sonic_rs::from_str::<sonic_rs::Value>(document).unwrap();
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
fn bench_sonic_rs(_document: &str) {}

fn bench_parse_small(c: &mut Criterion) {
    let mut group = c.benchmark_group("json/parse_small");
    group.bench_function("json", |b| b.iter(|| rs_json::parse(SMALL_OBJECT).unwrap()));
    group.bench_function("serde_json", |b| {
        b.iter(|| serde_json::from_str::<serde_json::Value>(SMALL_OBJECT).unwrap());
    });
    group.bench_function("legacy_json", |b| {
        b.iter(|| legacy_json::parse(SMALL_OBJECT).unwrap());
    });
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    group.bench_function("simd_json", |b| b.iter(|| bench_simd_json(SMALL_OBJECT)));
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    group.bench_function("sonic_rs", |b| b.iter(|| bench_sonic_rs(SMALL_OBJECT)));
    group.finish();
}

fn bench_parse_nested(c: &mut Criterion) {
    let mut group = c.benchmark_group("json/parse_nested");
    group.bench_function("json", |b| {
        b.iter(|| rs_json::parse(NESTED_OBJECT).unwrap());
    });
    group.bench_function("serde_json", |b| {
        b.iter(|| serde_json::from_str::<serde_json::Value>(NESTED_OBJECT).unwrap());
    });
    group.bench_function("legacy_json", |b| {
        b.iter(|| legacy_json::parse(NESTED_OBJECT).unwrap());
    });
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    group.bench_function("simd_json", |b| b.iter(|| bench_simd_json(NESTED_OBJECT)));
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    group.bench_function("sonic_rs", |b| b.iter(|| bench_sonic_rs(NESTED_OBJECT)));
    group.finish();
}

fn bench_parse_string_heavy(c: &mut Criterion) {
    let mut group = c.benchmark_group("json/parse_string_heavy");
    group.bench_function("json", |b| {
        b.iter(|| rs_json::parse(STRING_HEAVY_OBJECT).unwrap());
    });
    group.bench_function("serde_json", |b| {
        b.iter(|| serde_json::from_str::<serde_json::Value>(STRING_HEAVY_OBJECT).unwrap());
    });
    group.bench_function("legacy_json", |b| {
        b.iter(|| legacy_json::parse(STRING_HEAVY_OBJECT).unwrap());
    });
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    group.bench_function("simd_json", |b| {
        b.iter(|| bench_simd_json(STRING_HEAVY_OBJECT));
    });
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    group.bench_function("sonic_rs", |b| {
        b.iter(|| bench_sonic_rs(STRING_HEAVY_OBJECT));
    });
    group.finish();
}

fn bench_parse_number_heavy(c: &mut Criterion) {
    let mut group = c.benchmark_group("json/parse_number_heavy");
    group.bench_function("json", |b| {
        b.iter(|| rs_json::parse(NUMBER_HEAVY_OBJECT).unwrap());
    });
    group.bench_function("serde_json", |b| {
        b.iter(|| serde_json::from_str::<serde_json::Value>(NUMBER_HEAVY_OBJECT).unwrap());
    });
    group.bench_function("legacy_json", |b| {
        b.iter(|| legacy_json::parse(NUMBER_HEAVY_OBJECT).unwrap());
    });
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    group.bench_function("simd_json", |b| {
        b.iter(|| bench_simd_json(NUMBER_HEAVY_OBJECT));
    });
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    group.bench_function("sonic_rs", |b| {
        b.iter(|| bench_sonic_rs(NUMBER_HEAVY_OBJECT));
    });
    group.finish();
}

fn bench_repeated_key_traversal(c: &mut Criterion) {
    let parsed_json = rs_json::parse(NESTED_OBJECT).unwrap();
    let parsed_serde = serde_json::from_str::<serde_json::Value>(NESTED_OBJECT).unwrap();
    let parsed_legacy = legacy_json::parse(NESTED_OBJECT).unwrap();
    let mut group = c.benchmark_group("json/repeated_key_traversal");
    group.bench_function("json", |b| {
        b.iter(|| {
            for _ in 0..32 {
                black_box(parsed_json["config"]["enabled"].as_bool());
                black_box(parsed_json["meta"]["versions"][2].as_i128());
                black_box(parsed_json["users"][1]["name"].as_str());
            }
        });
    });
    group.bench_function("serde_json", |b| {
        b.iter(|| {
            for _ in 0..32 {
                black_box(parsed_serde["config"]["enabled"].as_bool());
                black_box(parsed_serde["meta"]["versions"][2].as_i64());
                black_box(parsed_serde["users"][1]["name"].as_str());
            }
        });
    });
    group.bench_function("legacy_json", |b| {
        b.iter(|| {
            for _ in 0..32 {
                black_box(parsed_legacy["config"]["enabled"].as_bool());
                black_box(parsed_legacy["meta"]["versions"][2].as_i64());
                black_box(parsed_legacy["users"][1]["name"].as_str());
            }
        });
    });
    group.finish();
}

fn bench_invalid(c: &mut Criterion) {
    let mut group = c.benchmark_group("json/invalid");
    group.bench_function("json", |b| {
        b.iter(|| rs_json::parse(INVALID_DOCUMENT).unwrap_err());
    });
    group.bench_function("serde_json", |b| {
        b.iter(|| serde_json::from_str::<serde_json::Value>(INVALID_DOCUMENT).unwrap_err());
    });
    group.bench_function("legacy_json", |b| {
        b.iter(|| legacy_json::parse(INVALID_DOCUMENT).unwrap_err());
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_parse_small,
    bench_parse_nested,
    bench_parse_string_heavy,
    bench_parse_number_heavy,
    bench_repeated_key_traversal,
    bench_invalid
);
criterion_main!(benches);
