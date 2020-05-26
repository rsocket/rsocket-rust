use criterion::criterion_main;

mod benchmarks;

criterion_main! {
    benchmarks::fibonaccis::benches,
    benchmarks::frames::benches,
}
