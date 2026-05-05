## Why

The status command is designed to aggregate git status across many repositories in parallel, but the project has no benchmark that measures the end-user command latency at realistic scale. A benchmark against 50 real-world repositories will make performance regressions visible as the command evolves.

## What Changes

- Add an opt-in benchmark for full `git-vmr status` command wall time against a VMR fixture containing 50 known real-world repositories.
- Have the benchmark clone the fixture repositories only when missing, then reuse them for future runs.
- Warm the filesystem and git metadata cache before measured iterations so results represent warm-cache steady-state performance.
- Keep benchmark fixture data outside tracked source files, under an ignored build/cache directory by default.
- Document how to run the benchmark through the repository's Nix development shell.

## Capabilities

### New Capabilities
- `status-benchmark`: Benchmark full status command wall time against a reusable 50-repository real-world fixture.

### Modified Capabilities

None.

## Impact

- Adds benchmark harness files under `benches/`.
- Adds benchmark-only dependencies and Cargo benchmark configuration.
- Uses network access during first benchmark setup to clone known repositories.
- Uses local disk space under `target/` or an override directory to retain cloned benchmark fixtures between runs.
