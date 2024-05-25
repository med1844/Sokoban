use criterion::{criterion_group, criterion_main, Criterion};
use sokoban::game::{board::Board, solver::Solver};
use std::fs;

pub fn benchmark(c: &mut Criterion) {
    let raw_level_1 = fs::read_to_string("levels/cognitive/1.txt").expect("Should have file");
    let board_1 = Board::from(raw_level_1.as_str());
    let mut group_1 = c.benchmark_group("solve cognitive 1");
    group_1.measurement_time(std::time::Duration::new(10, 0));
    group_1.sample_size(100);
    group_1.bench_function("solve cognitive 1", |b| {
        b.iter(|| {
            let solver_1 = Solver::new(board_1.clone());
            let _ = solver_1.solve(None);
        })
    });
    group_1.finish();

    let raw_level_2 = fs::read_to_string("levels/cognitive/2.txt").expect("Should have file");
    let board_2 = Board::from(raw_level_2.as_str());
    let mut group_2 = c.benchmark_group("solve cognitive 2");
    group_2.measurement_time(std::time::Duration::new(10, 0));
    group_2.sample_size(100);
    group_2.bench_function("solve cognitive 2", |b| {
        b.iter(|| {
            let solver_2 = Solver::new(board_2.clone());
            let _ = solver_2.solve(None);
        })
    });
    group_2.finish();

    let raw_level_3 = fs::read_to_string("levels/cognitive/3.txt").expect("Should have file");
    let board_3 = Board::from(raw_level_3.as_str());
    let mut group_3 = c.benchmark_group("solve cognitive 3");
    group_3.measurement_time(std::time::Duration::new(20, 0));
    group_3.sample_size(10);
    group_3.bench_function("solve cognitive 3", |b| {
        b.iter(|| {
            let solver_3 = Solver::new(board_3.clone());
            let _ = solver_3.solve(None);
        })
    });
    group_3.finish();

    let raw_level_4 = fs::read_to_string("levels/cognitive/4.txt").expect("Should have file");
    let board_4 = Board::from(raw_level_4.as_str());
    let mut group_4 = c.benchmark_group("solve cognitive 4");
    group_4.measurement_time(std::time::Duration::new(40, 0));
    group_4.sample_size(10);
    group_4.bench_function("solve cognitive 4", |b| {
        b.iter(|| {
            let solver_4 = Solver::new(board_4.clone());
            let _ = solver_4.solve(None);
        })
    });
    group_4.finish();
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
