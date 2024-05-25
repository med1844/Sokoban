# Sokoban

A command-line sokoban game where you can push multiple lined-up boxes. It also features a State-space A* solver with simple deadlock detection.

![demo](demo.gif)

## How to play

```fish
cargo run --release
```

- Use `q` to go back to previous screen
- Use arrow keys to move
- In game screen, press `o` to start solver

## Benchmarking

```fish
cargo bench
```

## Profiling

```fish
CARGO_PROFILE_BENCH_DEBUG=true cargo flamegraph --bench solver_benchmark
```

## Experimental solver features

### Freeze deadlock detection

Freeze deadlock detection was implemented but not enabled by default, due to the reduction in visited state shadowed by the overhead it introduced. You may run the game with `cargo r --release --features freeze_deadlock_check`. In `levels/cognitive/4.txt`, it reduced number of visited state from `117885` to `110204`.

### Bi-directional A* search

There doesn't seems to have anyone implemented this, so I gave it a try. Turns out it almost doubles visited state (probably bug), and almost doubles runtime. If you are interested in looking it into detail (don't do it, it's a total mess), go `git checkout bi_a_star`.

