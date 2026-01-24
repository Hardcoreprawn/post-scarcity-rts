//! Simulation benchmarks for rts_core.
//!
//! Run with: `cargo bench -p rts_core`

// Benchmark binaries don't need docs on macro-generated functions
#![allow(missing_docs)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};

/// Runs simulation benchmarks for the rts_core crate.
pub fn simulation_benchmark(c: &mut Criterion) {
    c.bench_function("placeholder", |b| {
        b.iter(|| {
            // TODO: Add actual simulation benchmarks
            black_box(1 + 1)
        })
    });
}

criterion_group!(benches, simulation_benchmark);
criterion_main!(benches);
