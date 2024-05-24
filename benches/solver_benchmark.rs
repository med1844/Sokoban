use criterion::{criterion_group, criterion_main, Criterion};
use sokoban::game::{board::Board, solver::Solver};
use std::fs;

pub fn benchmark(c: &mut Criterion) {
    let raw_level = fs::read_to_string("levels/cognitive/3.txt").expect("Should have file");
    let board = Board::from(raw_level.as_str());
    let mut group = c.benchmark_group("solve cognitive 3");
    group.measurement_time(std::time::Duration::new(20, 0));
    group.sample_size(10);
    group.bench_function("solve cognitive 3", |b| {
        b.iter(|| {
            let s = Solver::new(&board);
            let _ = s.solve(None);
        })
    });
    group.finish();
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
