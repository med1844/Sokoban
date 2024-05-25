# Sokoban

A command-line sokoban game, features a state-space A* solver with simple deadlock detection.

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

Freeze deadlock detection was implemented but not enabled by default, as the decrease in the number of visited states, while reducing runtime, did not sufficiently offset the additional overhead it introduced. You may run the game with `cargo r --release --features freeze_deadlock_check`. In `levels/cognitive/4.txt`, it reduced number of visited state from `117885` to `110204`.

### Bi-directional A* search

There doesn’t seem to be anyone who has implemented this, so I gave it a try. It turns out it almost doubles the number of visited states (probably a bug) and nearly doubles the runtime. If you're interested in looking into it in detail (don't do it, it's a total mess), go `git checkout bi_a_star`.

