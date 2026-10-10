---
status: accepted
---

# Result aggregation is told its scope; the scope is never derived from outcomes

Result aggregation (`render::outcomes`) takes the child repos in scope as an explicit input: an iterator of repo names, duplicates allowed, deduplicated and counted inside aggregation. The repo-list suffix's "no suffix means everyone" denominator is the number of distinct names in that scope. `Workspace::run`/`run_routed` pass the repos they fan over; `mv` passes one name per routed source plus the destination.

The scope cannot be reconstructed from the outcomes themselves: quiet successes (`RepoOutcome::Success(None)`) and hard errors carry no repo name, and a repo in scope may produce no outcome at all — a move's destination repo is named only when staging the addition fails. Deriving the denominator from the repo names present in the outcomes therefore undercounts, and a failure confined to one repo would falsely render as applying to everyone.

## Considered Options

- **Derive the denominator from distinct repo names in the outcomes** — rejected: unsound, per the anonymity argument above. Example: sources in repos A and B moving to C, where A's entry fails and B's succeeds quietly — the outcomes name only {A}, so the derived denominator is 1 and the "(A)" suffix is wrongly omitted.
- **Tag every outcome with the repos it speaks for, and union the tags** — rejected: reshapes `GitCommandResult` across every slice to carry a repo set that is a singleton everywhere except move entries, and still needs the destination attached to outcomes that never touched it.
- **Keep a separate `outcomes_in_scope(results, count)` entry point** (the previous shape) — rejected: callers do rendering's math; `mv` maintained its own `HashSet` to compute the count, and the two entry points let a caller pick the wrong one.

## Consequences

- One aggregation entry point; `outcomes_in_scope` and `mv`'s scope-counting `HashSet` are deleted.
- The denominator rules (duplicate scope names deduplicate, scope repos without outcomes widen the denominator, quiet successes count via scope) are unit-tested at the aggregation seam; the integration suite remains the behavior-preservation guard.
- One deliberate rendering change: `worktree remove --delete` produces two outcomes per child repo, so its old denominator (the outcome count) meant branch-deletion failures always carried a repo-list suffix. With the honest scope, a failure covering every repo in scope now drops its suffix — "no suffix means everyone", uniformly with every other command.
- A future command whose outcome count differs from its repo count needs no new aggregation machinery — it passes its scope like everyone else.
