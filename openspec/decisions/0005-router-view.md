---
status: accepted
---

# Routing is a private borrowed view behind the workspace

Routing lives in one module, `workspace/routing.rs`: a `Router` holding borrowed references to the workspace's fixed snapshot — the VMR root, the child repos, and the working dir the workspace was opened from. Its interface is three methods: `route(scope)` groups owning child repos with repo-relative paths, `route_single(path)` routes one operand (always denying the aggregate path), and `target(path)` normalizes a user path against the captured working dir. The aggregate-path predicate is defined once, inside the Router. The workspace stores the working dir at `find` time and constructs the Router on demand; slices reach routing only through the workspace's `run_routed`, `route_single`, and `target`.

## Considered Options

- **Publish the Router to the slices** (`workspace.router()` public) — rejected: the Router has exactly one production caller (the workspace), so publishing creates a second way to route with no second adapter to justify it, and every routing consumer also wants the execute-and-render half that `run_routed` already hides.
- **Router owns the snapshot**, with the workspace delegating `repos()`/`root()` inward — rejected: it puts routing in the middle of concerns that have nothing to do with it (parallel execution, rendering), and Rust rules out the workspace holding a Router borrowing its own fields anyway.
- **Working dir passed per call** (the previous shape) — rejected: the parameter could only ever legitimately hold the value the workspace was opened with, so it was pure interface width, and a different value was a bug the signatures invited. Capturing it also let `resolve_target` leave the public surface: `workspace.target(path)` replaces the free function for the worktree slice's non-routing target normalization.

## Consequences

- Routing behavior — aggregate expansion, policy denial, ownership checks, `.gitvmr` rejection, repo-relative rebasing, target normalization — is unit-tested at the Router seam as pure path math: literal path fixtures, no filesystem, no scripted fake. The workspace keeps thin composition tests for `run_routed` through the scripted fake; the integration suite stays the real-git guard (ADR-0001).
- `Vmr` returns to root discovery, marker lifecycle, and repo scanning; it no longer knows how paths route.
- `Workspace::route_single` and `Workspace::target` are deliberate one-line pass-throughs: deleting them would force slices to construct Routers and know an internal seam of the workspace core.
- `Scope` and `AggregatePolicy` live with the Router and are re-exported from `crate::workspace`, keeping slice imports stable (ADR-0003's slices-depend-on-cores rule unchanged).
