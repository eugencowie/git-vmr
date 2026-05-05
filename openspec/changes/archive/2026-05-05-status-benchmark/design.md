## Context

`git-vmr status` discovers immediate child repositories under a VMR root, collects each repository's status in parallel with rayon, and renders one combined status view. Existing tests cover correctness with small synthetic repositories, but there is no performance benchmark for the real user-facing command against a large VMR.

The benchmark needs to measure full command wall time, including process startup, VMR discovery, repository discovery, `gix` status collection, rendering, and output writes. Fixture setup must be excluded from measured timing, but the benchmark should be able to prepare its own fixture the first time it runs.

## Goals / Non-Goals

**Goals:**
- Measure full `git-vmr -C <fixture> status` wall-clock latency as a subprocess.
- Benchmark against 50 known real-world git repositories.
- Clone benchmark repositories once and reuse them on later benchmark runs.
- Use warm-cache measurements by running unmeasured warmup invocations before sampling.
- Keep cloned repositories out of source control and away from normal test fixtures.
- Make the benchmark opt-in so normal `cargo test` remains fast and offline.

**Non-Goals:**
- Benchmark only the internal `collect_statuses` function.
- Include repository clone/setup time in measured status latency.
- Auto-update or pull benchmark repositories after initial setup.
- Require the benchmark to run in CI by default.
- Define a pass/fail performance threshold in this change.

## Decisions

### Decision: Use a Cargo benchmark target with a subprocess harness

The benchmark will live under `benches/` and invoke the compiled `git-vmr` binary as a subprocess with `-C <fixture-root> status`.

Alternatives considered:
- **Internal function benchmark**: cleaner microbenchmark signal, but it excludes process startup, CLI parsing, stdout behavior, and other user-visible costs.
- **Shell script with `hyperfine`**: good command benchmarking ergonomics, but it would add an external tool requirement and sit outside the Rust/Cargo workflow.

Subprocess timing best matches the requested full command wall time.

### Decision: Keep fixtures under `target/bench-fixtures` by default

The default fixture root will be inside the ignored `target/` tree, for example `target/bench-fixtures/status-50`. The benchmark may support an override such as `GIT_VMR_BENCH_FIXTURE_DIR` for users who want the clones elsewhere.

Alternatives considered:
- **Repository-local fixture directory**: easier to find, but risks accidental source control churn and pollutes the workspace.
- **System cache directory**: survives `cargo clean`, but is less transparent and varies by platform.

`target/` matches Rust developer expectations for generated, disposable artifacts.

### Decision: Use a pinned repository manifest

The benchmark will define 50 repository entries with stable names, clone URLs, and pinned revisions. On first setup, the harness clones missing repositories and checks out the pinned revision. On later runs, existing repositories are reused without pulling.

Alternatives considered:
- **Clone current default branches**: simpler, but benchmark inputs drift over time and differ by setup date.
- **Synthetic generated repositories**: reproducible and cheap, but does not exercise real repository structures.

Pinned real repositories preserve realistic working trees while keeping benchmark inputs stable.

### Decision: Prepare a valid VMR fixture root

The fixture root will contain a `.gitvmr/` marker and 50 immediate child git repositories. This mirrors the production status command's discovery path instead of bypassing it.

The benchmark should validate the fixture before timing and fail with a clear error if setup is incomplete or corrupted.

### Decision: Warm cache before measuring

Before measured samples, the harness will run `git-vmr -C <fixture-root> status` one or more times and discard the results. Measured iterations will also discard stdout/stderr to avoid terminal rendering cost and noisy benchmark output.

This measures steady-state warm-cache latency. Cold-cache behavior is machine and OS dependent and is out of scope for this benchmark.

### Decision: Do not update retained clones automatically

If a repository directory already exists, the benchmark will not fetch or pull. If the existing checkout is not at the expected revision, the harness may either check out the pinned revision or report a clear fixture validation error.

Avoiding network access on subsequent runs keeps benchmark execution predictable after initial setup.

## Risks / Trade-offs

- Network setup can fail during first run -> keep setup outside measured samples and report clone failures clearly.
- Public repositories can disappear or rewrite history -> use stable, well-known repositories and pinned revisions, and keep the manifest easy to update.
- Fixture consumes significant disk space -> store under `target/` and document that `cargo clean` or deleting the fixture removes it.
- Warm-cache results hide first-run latency -> document that the benchmark intentionally measures warm-cache steady state.
- Subprocess benchmarks include process startup noise -> accept this because full command wall time is the target measurement.
- Large benchmark is slow compared with tests -> make it opt-in via `cargo bench`, not part of normal test execution.
