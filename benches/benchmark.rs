mod static_arrays;
use criterion::{criterion_group, criterion_main};

criterion_group!(benches, static_arrays::bench_static_arrays);
criterion_main!(benches);
