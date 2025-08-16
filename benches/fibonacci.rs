use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

use layer8_interceptor_production::fibonacci;

fn fibonacci_benchmark<const INPUT: u64>(c: &mut Criterion) {
    c.bench_function(&format!("fib {INPUT}"), |b| {
        b.iter(|| fibonacci(black_box(INPUT)))
    });
}

criterion_group!(
    benches,
    fibonacci_benchmark<0>,
    fibonacci_benchmark<1>,
    fibonacci_benchmark<2>,
    fibonacci_benchmark<8>,
    fibonacci_benchmark<16>,
    fibonacci_benchmark<32>,
);

criterion_main!(benches);
