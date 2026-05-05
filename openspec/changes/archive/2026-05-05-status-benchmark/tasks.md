## 1. Benchmark Configuration

- [x] 1.1 Add Cargo benchmark configuration for a `status_50_real_repos` benchmark target.
- [x] 1.2 Add benchmark-only dependencies needed for wall-clock command timing and temporary/setup utilities.
- [x] 1.3 Define the default benchmark fixture location under `target/bench-fixtures/status-50`.
- [x] 1.4 Add an optional environment variable override for the benchmark fixture directory.

## 2. Repository Manifest

- [x] 2.1 Create a stable manifest of 50 known real-world repositories with names, clone URLs, and pinned revisions.
- [x] 2.2 Ensure manifest repository names are valid immediate child directory names for a VMR root.
- [x] 2.3 Prefer repositories with varied sizes and file layouts so the fixture exercises realistic status traversal.

## 3. Fixture Setup

- [x] 3.1 Create benchmark setup logic that creates the fixture root and `.gitvmr/` marker when missing.
- [x] 3.2 Clone any configured repository that is missing from the fixture.
- [x] 3.3 Check out each newly cloned repository at its configured pinned revision.
- [x] 3.4 Reuse existing repository directories without fetching or pulling from the network.
- [x] 3.5 Validate that the fixture contains `.gitvmr/` and all 50 configured immediate child git repositories before measurement.
- [x] 3.6 Emit clear setup errors that identify the repository or fixture condition that failed.

## 4. Command Benchmark

- [x] 4.1 Locate the compiled `git-vmr` binary for benchmark execution.
- [x] 4.2 Execute measured samples as `git-vmr -C <fixture-root> status` subprocesses.
- [x] 4.3 Exclude all fixture setup and validation work from measured samples.
- [x] 4.4 Discard or capture stdout and stderr for warmup and measured command runs.
- [x] 4.5 Fail the benchmark if any status command invocation exits unsuccessfully.

## 5. Warm-Cache Measurement

- [x] 5.1 Run at least one unmeasured status command invocation after fixture validation and before measured samples.
- [x] 5.2 Configure the benchmark name/output to make clear that it reports warm-cache full-command wall time.
- [x] 5.3 Avoid cold-cache claims or pass/fail performance thresholds in this change.

## 6. Documentation and Verification

- [x] 6.1 Document how to run the benchmark through `nix develop -c cargo bench --bench status_50_real_repos`.
- [x] 6.2 Document that the first run clones repositories and later runs reuse the retained fixture.
- [x] 6.3 Document how to remove or relocate the retained fixture.
- [x] 6.4 Verify the benchmark compiles.
- [x] 6.5 Verify a setup path creates or validates a fixture without including setup time in measured samples.
- [x] 6.6 Run `nix develop -c openspec status --change status-benchmark` and confirm the change is apply-ready.
