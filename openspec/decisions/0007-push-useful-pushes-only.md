---
status: accepted
---

# Push attempts only provably useful pushes

`git vmr push` was a verbatim arg-passer: every child repo got the user's arguments untouched, and all repository-state judgment was delegated to Git (the push spec explicitly forbade pre-filtering on branch state). This polluted remotes in the core VMR workflow — create a feature branch across the workspace, commit in two or three repos, push — by creating empty branches (and firing CI) in every repo with nothing new. We reversed the delegation requirement: push now attempts only useful pushes — those proven, from local remote-tracking refs alone, to transmit novel commits or to update or delete a ref that already exists on the target remote — and skips everything unproven, reporting each skip with its reason.

The governing asymmetry: a wrong push creates an empty branch and triggers CI while looking like an ordinary success — invisible; a wrong skip is reported, visible, and recoverable (`git vmr foreach 'git push'` is the deliberate fallback — there is no override flag). So the filter errs toward skipping, not pushing. Evidence is local-only — never the network — so skipped repos cost nothing; a stale picture degrades to a redundant push that Git absorbs.

## Considered Options

- **Keep full delegation, filter only bare `git vmr push`** — rejected: the filter is the standard behavior users should experience on every form of push, including explicit refspecs; escape hatches breed the exact pollution the feature removes.
- **"When in doubt, push"** (skip only positively-proven-empty creations) — rejected: the doubt cases are where pollution hides (e.g. a branch merged and deleted on the remote but still carrying local upstream config is silently re-created by a bare push).
- **Ask the remote** (`ls-remote` / `--dry-run`) for an accurate picture — rejected: it makes every skipped repo as expensive as a push, defeating the point.
- **Repo-level granularity for mixed refspec lists** — rejected in favor of per-refspec filtering: each repo pushes only its useful subset of the typed refspecs, so pollution cannot leak through a mixed command line. Bare push is the one-refspec degenerate case of the same mechanism.

## Consequences

- git-vmr is no longer a verbatim arg-passer: what runs in a child repo is a computed subset of what the user typed. The push spec's delegation requirement is rewritten accordingly.
- The usefulness test applies to branch destinations only — the sole ref class whose creation constitutes pollution. Singular non-branch refs (tags, deletes) pass on explicit intent (tag existence is locally unverifiable, and a tag's commit pre-existing on the remote is the normal case); wildcard refspecs skip as unclassifiable; a URL repository argument delegates the whole push (no local picture, strongest intent).
- Repo outcome gains a third variant, skipped, with a required reason — aggregated and repo-suffixed like the others, printed to stdout, never affecting the exit code. An all-skipped run exits 0 and is not silent.
- Detached and unborn heads may be delegated or skipped, whichever the implementation makes simplest; Git's repo-suffixed errors are acceptable there.
