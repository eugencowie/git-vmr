---
status: accepted
---

# The run pipeline owns the invocation lifecycle behind one seam

Everything between a parsed CLI and the process exit code is one module: the run pipeline (`cli/pipeline.rs`). Its interface follows the crate's function-pair convention — `run(cli)` wires the production adapters (resolved global config and state paths, the process stderr, `analytics::record`, `updates::check`), and `run_with(...)` is the test surface, taking the paths, a stderr writer, a recorder shaped like `analytics::record`, and a checker shaped like `updates::check`. The pipeline's rules — load warnings print before the command runs, the recorder sees the command's success and duration, the update notice follows the command's output, save failures warn without changing the exit code — are unit-tested at this seam. `Cli::run` is delegation only.

## Considered Options

- **Reach past the recorder/checker to the inner sink and query seams** (`record_with_sink`, `check_with_query`) — rejected: the pipeline would absorb sink selection and need `now` on its interface, and its tests would re-cover session rotation and due-ness already unit-tested where they live. The analytics-sink seam stays an analytics-internal concern.
- **The pipeline owns all printing**, absorbing `render::emit`'s choke point behind injected stdout+stderr writers — rejected: it reshapes `render::emit` and widens the interface by two writers to unit-test cross-stream ordering the integration suite already guards.
- **Extract only the post-command tail** (record → check → save) — rejected: the ordering rules span execution (warnings first, notice after output, exit code mirrors the command), so a tail-only module leaves `Cli::run` a weld of execute + tail and the spanning rules integration-only.
- **Inject the git adapter into `run_with`** — rejected: the pipeline treats the command as opaque and its rules never observe git variance; its tests use commands that never spawn git. Scripted-fake territory stays with the slices (ADR-0001).

## Consequences

- The three notice prints that sat outside `render::emit`'s choke point (`warning:` lines, the update notice) move behind the pipeline's stderr writer; `main.rs` keeps the final error print.
- `tests/analytics.rs` and `tests/updates.rs` stay as the real-binary guard and must not be mass-ported onto the pipeline seam (per ADR-0001's discipline).
- `cli.rs` shrinks to parsing, metadata derivation, and delegation; it no longer imports `analytics`, `updates`, or `render`.
