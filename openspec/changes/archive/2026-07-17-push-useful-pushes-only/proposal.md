## Why

`git vmr push` passes the user's arguments verbatim to every child repo, so the core VMR workflow — create a feature branch across the workspace, commit in two or three repos, push — creates empty branches on the remotes of every repo with nothing new, polluting them and firing CI for nothing. A wrong push is invisible (it looks like success); a wrong skip is visible and recoverable, so push should err toward skipping.

## What Changes

- **BREAKING** `git vmr push` attempts only *useful pushes*: those proven, from local remote-tracking refs alone (never the network), to transmit novel commits or to update or delete a ref that already exists on the target remote. Everything unproven is skipped with a reported reason. This reverses the push spec's "no pre-filtering on branch state" delegation requirement (see ADR `openspec/decisions/0007-push-useful-pushes-only.md`).
- The filter applies to every form of the command — bare, with a remote, with explicit refspecs — with no override flag; `git vmr foreach 'git push'` is the fallback.
- Filtering is per refspec: each repo pushes only its useful subset of the typed refspecs; a repo with none gets no git invocation. Bare push is the one-refspec degenerate case.
- Refspec classification: branch refspecs get the usefulness test; deletes are always useful; tags always push (locally unverifiable, explicit intent); wildcards skip as unclassifiable; a URL repository argument delegates the whole push untouched.
- Repo outcome gains a third variant, *skipped*, with a required reason — aggregated and repo-suffixed like the others, printed to stdout, never affecting the exit code. An all-skipped run exits 0 and is not silent.
- Detached and unborn heads may be delegated or skipped, whichever is simplest.

## Capabilities

### New Capabilities

_None — the behavior belongs to existing capabilities._

### Modified Capabilities

- `push-command`: the "delegates Git argument and repository-state resolution" requirement is replaced by the useful-push filter; new requirements cover per-refspec filtering, refspec classification, and skip reporting.
- `aggregate-command-output`: repo outcome gains the skipped variant, flowing through result aggregation and rendering.

## Impact

- `src/commands/push.rs` — the filter, refspec classification, and target-remote/destination resolution.
- `src/git/` (repo outcome, report policy, result aggregation) and `src/render/` — the skipped variant and its (dimmed) rendering.
- `CONTEXT.md` glossary terms (useful push, novel commits, empty branch, skipped) and ADR 0007 already record the decisions.
- Integration tests for push; existing scenarios asserting verbatim delegation are rewritten.
