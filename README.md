# Sokoban

A command-line sokoban game where you can push multiple lined-up boxes. It also features a solver.

## Benchmarking

```fish
cargo bench
```

## Profiling

```fish
CARGO_PROFILE_BENCH_DEBUG=true cargo flamegraph --bench solver_benchmark
```
