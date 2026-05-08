## Context

`git-vmr` operates on immediate child Git repositories under a virtual monorepo root marked by `.gitvmr/`. Existing commands discover the VMR root from the effective working directory, skip non-Git child directories, shell out to Git for repository semantics, and report repository names when aggregating failures.

`git vmr branch <branch-name>` already establishes a parallel best-effort mutating pattern: attempt the operation in every child Git repository, collect failures, and return a non-zero status if any repository fails. `git vmr merge` establishes the related no-rollback model for history-changing Git operations. `git vmr rebase <upstream>` should combine those ideas: run per-repository Git rebases in parallel, do not roll back successful repositories, and preserve any in-progress rebase state left by Git.

## Goals / Non-Goals

**Goals:**

- Add `git vmr rebase <upstream>` for rebasing every immediate child Git repository onto the same Git upstream expression.
- Preserve normal Git rebase behavior in each child repository, including fast-forwards, replayed commits, already-up-to-date results, missing upstream failures, hooks, and conflicts.
- Attempt every discovered child Git repository even when one or more repositories fail.
- Run child repository rebase attempts in parallel.
- Succeed quietly when all attempted rebases succeed.
- Report per-repository failures deterministically.
- Reuse the existing global `-C <path>` and VMR root discovery behavior.

**Non-Goals:**

- Do not add rebase lifecycle modes such as `--abort`, `--continue`, `--skip`, or `--quit`.
- Do not add rebase mode flags such as `--onto`, `--interactive`, `--rebase-merges`, `--autostash`, strategy options, or generic passthroughs in the initial command.
- Do not pre-resolve or synchronize upstream refs across repositories.
- Do not roll back successful rebases if another repository fails.
- Do not automatically abort conflicted or interrupted rebases.
- Do not support recursive repository discovery or configured repository inclusion/exclusion.

## Decisions

### Decision: Delegate rebase semantics directly to Git

The command should shell out to `git --no-optional-locks -C <repo> rebase <upstream>` for each child repository. This keeps behavior aligned with users' Git expectations and avoids reimplementing upstream resolution, patch replay, conflict handling, hooks, or rebase state detection in Rust.

Alternative considered: inspect refs and repository state before running rebase. That could produce friendlier skip behavior, but it would diverge from Git's own validation and make edge cases such as tags, detached commits, remote-tracking names, and already-in-progress operations harder to preserve correctly.

### Decision: Treat every child Git repository as eligible

The command should attempt the rebase in every immediate child Git repository. A missing upstream or invalid revision in one repository is a failure for that repository, not a reason to skip it before invoking Git.

Alternative considered: skip repositories where the named upstream does not exist. That would be convenient for partially rolled out branches, but `git rebase <upstream>` accepts more than local branch names. Pre-filtering would incorrectly reject valid revisions and make the command less Git-like.

### Decision: Run rebases in parallel

The command should run child repository rebase attempts in parallel and capture each repository's output before rendering any aggregate failure report. Repositories are independent Git working trees, and parallel execution matches the requested best-effort behavior while reducing wall-clock time for VMRs with many children.

Alternative considered: run rebases sequentially like `merge`. Sequential execution is simpler to reason about, but it is not required if output is captured and rendered after all attempts complete.

### Decision: Quiet success, deterministic failure output

If all attempted rebases succeed, the command should produce no output. If any repository fails, stderr should contain one concise failure line per failed repository, ordered deterministically by repository name, using the first non-empty Git stderr line and falling back to stdout when stderr is empty.

Alternative considered: print success summaries for each rebased repository. That would mirror `merge`, but quiet success is more script-friendly and matches the requested command shape.

### Decision: Best-effort means no cross-repository transaction

If a rebase succeeds in one repository and fails in another, the successful rebase remains applied. If Git leaves a repository in an in-progress rebase state, `git-vmr` should leave that state intact and report the failure. Users can then resolve, continue, skip, or abort with normal Git commands inside the affected repositories.

Alternative considered: attempt automatic rollback on any failure. That is risky because a stopped rebase can contain useful conflict state, and automatic cleanup could discard work or fail in its own right.

## Risks / Trade-offs

- [Risk] Conflicted rebases leave repositories in in-progress rebase state -> Mitigation: specify and test this as intentional best-effort behavior.
- [Risk] The same `<upstream>` may resolve differently per repository -> Mitigation: delegate to Git and report per-repository failures rather than trying to normalize refs across repos.
- [Risk] Quiet success may hide which repositories were rewritten -> Mitigation: keep v1 script-friendly and rely on existing `git vmr status` and `git vmr branch` commands for follow-up inspection.
- [Risk] Parallel execution can make raw Git output interleaving unreadable -> Mitigation: capture per-repository output and render only deterministic aggregate failure lines after all attempts finish.
