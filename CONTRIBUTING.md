# Contributing

## Benchmarks

Run the real-repository benchmarks with:

```sh
cargo bench --bench real_repos
```

The first run prepares `target/bench-fixtures/real-repos` by cloning the pinned repository manifest and creating the `.gitvmr/` marker. Later runs reuse existing repository directories without fetching or pulling, so setup time is not part of the measured warm-cache full-command wall time.

This folder will consume approx. 5 GB of disk space. To keep the fixture somewhere else, set `GIT_VMR_BENCH_FIXTURE_DIR` to the target directory before running the benchmark. Remove the default fixture with:

```sh
rm -rf target/bench-fixtures/real-repos
```
