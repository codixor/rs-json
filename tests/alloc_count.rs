//! Check parser reuse in a single-threaded process with an instrumented allocator.

use rs_json::Parser;
use stats_alloc::{INSTRUMENTED_SYSTEM, Region, StatsAlloc};
use std::{alloc::System, hint::black_box};

#[path = "../benches/support/small_inputs.rs"]
mod small_inputs;

#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

fn main() {
    const REPEATS: usize = 1_000;
    let control = Region::new(GLOBAL);
    drop(black_box(Box::new(42_u64)));
    let control = control.change();
    assert!(
        control.allocations > 0 && control.deallocations > 0,
        "allocator control must see the boxed value"
    );

    for &(name, doc) in small_inputs::SAMPLES {
        let mut parser = Parser::new();
        let cold = Region::new(GLOBAL);
        black_box(parser.parse(black_box(doc)).unwrap());
        let cold = cold.change();

        let warm = Region::new(GLOBAL);
        for _ in 0..REPEATS {
            black_box(parser.parse(black_box(doc)).unwrap());
        }
        let warm = warm.change();
        assert_eq!(
            (warm.allocations, warm.reallocations, warm.deallocations),
            (0, 0, 0),
            "{name}: heap traffic after warmup"
        );
        println!(
            "{name}: {} cold allocation(s); {REPEATS} warm parses, {} allocations, {} reallocations, {} deallocations",
            cold.allocations, warm.allocations, warm.reallocations, warm.deallocations
        );
    }
    let changed: Vec<_> = small_inputs::SAMPLES
        .iter()
        .map(|&(n, s)| small_inputs::alternate(n, s))
        .collect();
    let mut parser = Parser::new();
    for &(_, s) in small_inputs::SAMPLES {
        black_box(parser.parse(black_box(s)).unwrap());
    }
    let warm = Region::new(GLOBAL);
    for _ in 0..REPEATS {
        for (&(_, s), other) in small_inputs::SAMPLES.iter().zip(&changed) {
            black_box(parser.parse(black_box(other)).unwrap());
            black_box(parser.parse(black_box(s)).unwrap());
        }
    }
    let stats = warm.change();
    assert_eq!(
        (stats.allocations, stats.reallocations, stats.deallocations),
        (0, 0, 0),
        "alternating payloads and shapes"
    );
    println!("mixed: 8,000 warm parses, 0 allocations, 0 reallocations, 0 deallocations");
}
