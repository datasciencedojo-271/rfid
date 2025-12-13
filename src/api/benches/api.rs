#![allow(missing_docs)]

use criterion::{criterion_group, criterion_main, Criterion};

fn bench_(c: &mut Criterion) {}

// link symbol to avoid the dead_code warning when clippy analyzes test targets.
const _: fn(&mut Criterion) = bench_;

criterion_group!(benches, bench_);
criterion_main!(benches);
