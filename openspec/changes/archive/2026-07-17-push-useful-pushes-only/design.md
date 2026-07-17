## Context

`git vmr push` (`src/commands/push.rs`) passes its arguments verbatim to `git push` in every child repo through the workspace fan-out. The grilling session behind this change resolved every behavioral decision; they are recorded in ADR `openspec/decisions/0007-push-useful-pushes-only.md` and as glossary terms in `CONTEXT.md` (useful push, novel commits, empty branch, skipped repo outcome). This document maps those decisions onto the codebase.

## Goals / Non-Goals

**Goals:**
- Classify each push per repo (and per refspec) as useful, skippable, or delegated, from local remote-tracking refs only.
- Introduce the skipped variant of `RepoOutcome` and carry it through result aggregation and rendering.
- Keep skipped repos zero-cost: no git subprocess where classification needs none, never any network.

**Non-Goals:**
- No suppression of `Everything up-to-date` on existing remote branches — those stay delegated.
- No override flag; `git vmr foreach 'git push'` is the fallback.
- No changes to other commands' behavior (pull/fetch symmetry is a possible follow-up, not this change).

## Decisions

- **Classification lives in the push slice.** The predicate is push-specific vocabulary; per ADR-0003 it belongs in `src/commands/push.rs`, not the workspace core. The workspace `run` fan-out gains nothing push-shaped.
- **Evidence gathering goes through the Git runner seam** (ADR-0001), so the scripted fake can script it. Candidate plumbing per repo: `git symbolic-ref -q HEAD` (head/branch), `git config` reads for `push.default`, `branch.<name>.remote`/`.pushRemote`, `remote.pushDefault` (target-remote and destination resolution), `git rev-parse --verify refs/remotes/<remote>/<dst>` (destination existence), and `git rev-list -1 <tip> --not --remotes=<remote>` (empty output ⇒ no novel commits). Exact commands may shift during implementation; the seam and the local-only constraint may not.
- **Destination resolution mirrors Git's, conservatively.** Resolve what `git push` would do for the common `push.default` values (`simple`, `current`, `upstream`) and upstream config; any configuration the resolver does not understand classifies as unproven → skip with reason. This is the "err toward skipping" principle from ADR 0007 applied to config space.
- **Refspec parsing is minimal**: split `+`/`src:dst`, detect `:dst` deletes, wildcards (`*`), and tag sources/destinations (resolve src via `rev-parse --symbolic-full-name`). Branch-shaped refspecs get the predicate; everything else follows the classification table in the spec delta.
- **`RepoOutcome::Skipped(reason)`** joins `Success`/`Failure` in `src/git`. Aggregation groups skips by reason like any message; rendering dims them; exit-code logic ignores them. The quiet-success rule explicitly does not apply to skips.
- **Detached/unborn heads**: whichever classification falls out simplest — delegation (Git's repo-suffixed error) is acceptable per the session.

## Risks / Trade-offs

- [Stale remote-tracking refs skip a wanted push — e.g. remote branch deleted but not pruned locally] → skips are always reported with reasons, and `git vmr foreach 'git push'` recovers; document `git fetch --prune` in the skip reason where apt.
- [Reimplementing `git push` destination resolution drifts from Git across versions/configs] → the resolver is conservative: anything unrecognized skips with a reason rather than guessing; the classification table is spec-tested scenario by scenario.
- [Third `RepoOutcome` variant touches shared machinery used by every fan-out command] → other commands simply never construct it; aggregation/rendering changes are additive and covered by the aggregate-command-output delta spec.
