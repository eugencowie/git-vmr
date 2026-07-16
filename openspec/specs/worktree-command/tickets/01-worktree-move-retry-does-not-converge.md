# worktree move retry does not converge after partial failure

Status: needs-triage

## Problem

When `worktree move` partially fails, the source and destination are both worktree roots holding a subset of the children. Rerunning the same move cannot heal this: the already-moved children fail ("source does not exist"), `workspace.run` reports the failure, and the source root is never dissolved — even once every child has in fact moved. The dissolve is gated on "this run fully succeeded" rather than on "the source root has no remaining child worktrees".

Current behavior is pinned by the unit test `partial_move_failure_keeps_both_roots_and_renders_successes` in `src/commands/worktree.rs`; that test documents the baseline this issue would change.

## Proposed change

Gate dissolving the source root on it having no remaining child worktrees (and tolerate per-child "already at destination" outcomes on retry), so a rerun after a partial failure converges to a single destination root. Needs its own design pass: what counts as a remaining child, and how a retried move should report children that already moved.

## Notes

Deferred from the mv/worktree deepening round (2026-07-07) — the round was scoped to pinning current behavior with scripted-fake tests, not changing materialize/dissolve semantics. Vocabulary: see CONTEXT.md (worktree root, materialize/dissolve, repo outcome).
