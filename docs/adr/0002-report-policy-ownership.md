---
status: accepted
---

# Report-policy choices live in each operation's file, and the policy owns its message transform

Each git operation declares its report policy in its own vertical file, next to its arg construction and its scripted-fake tests. The policy types (`SuccessReport`, `FailureReport`) are structs — a message source plus an optional transform — built with constructors and a `.map(fn)` combinator, and the engine (`command_result`) applies the transform to every message the policy emits. Declaring a transform is wiring it: post-processing outside the policy is not a thing. Operations that run the same git command share one named policy (`switch` and `create` both use `git switch` and share `report_policy()` in `switch.rs`).

## Considered Options

- **Central per-op policy table** (a `report/policies.rs` listing every operation's choices side by side, diffable on one screen) — rejected: it cuts across the vertical one-file-per-operation structure, needs functions rather than constants anyway (`rm` picks its success policy from `--dry-run`; branch delete formats runtime data into its message), and drifts toward the declarative operation registry ADR-0001 already rejected.
- **Transform as post-processing on the outcome** (`RepoOutcome::map_success_message`, the previous shape) — rejected: the transform sat outside the declared policy, so two call sites of the same git command could silently disagree — `switch` normalized its fast-forward message while `create` did not.

## Consequences

- Cross-operation policy differences (e.g. `fetch` reads stderr first, `pull` reads stdout first) are legitimate per-command facts, pinned by each operation's unit tests — not drift to be centralized away.
- `FailureReport.map` mirrors `SuccessReport.map` for symmetry but has no production caller yet; it carries a scoped dead-code expectation until a failure transform appears.
- Policy wiring is guarded by scripted-fake unit tests; the integration suite stays the real-git guard and must not be mass-ported (per ADR-0001).
