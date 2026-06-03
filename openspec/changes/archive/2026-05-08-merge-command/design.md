## Context

`git-vmr` operates on immediate child Git repositories under a virtual monorepo root marked by `.gitvmr/`. Existing repo-wide commands discover the VMR root from the effective working directory, skip non-Git child directories, shell out to Git for repository semantics, and either render aggregate information or attempt mutating operations across child repositories with deterministic reporting.

`git vmr branch <branch-name>` already establishes the best-effort mutating pattern: attempt the operation in every child Git repository, collect failures, and return a non-zero status if any repository fails. `git vmr merge` should use the same model because a merge can succeed in some repositories while failing in others due to conflicts, missing refs, or repository-specific state.

## Goals / Non-Goals

**Goals:**
- Add `git vmr merge <commit-ish>` for merging the same Git revision expression in every immediate child Git repository.
- Preserve normal Git merge behavior in each child repository, including fast-forwards, merge commits, already-up-to-date results, conflicts, hooks, and Git's own error messages.
- Attempt every discovered child Git repository even when earlier repositories fail.
- Report per-repository success and failure output deterministically.
- Reuse the existing global `-C <path>` and VMR root discovery behavior.

**Non-Goals:**
- Do not add merge mode flags such as `--abort`, `--continue`, `--ff-only`, `--no-ff`, `--no-commit`, strategy options, or strategy-option passthroughs in the initial command.
- Do not pre-resolve or synchronize refs across repositories.
- Do not roll back successful merges if another repository fails.
- Do not automatically abort conflicted merges.
- Do not support recursive repository discovery or configured repository inclusion/exclusion.

## Decisions

### Decision: Delegate merge semantics directly to Git

The command should shell out to `git --no-optional-locks -C <repo> merge <commit-ish>` for each child repository. This keeps behavior aligned with users' Git expectations and avoids reimplementing merge analysis in Rust.

Alternative considered: inspect refs and repository state before running merge. That could produce friendlier skip behavior, but it would diverge from Git's own validation and make edge cases such as tags, detached commits, remote-tracking names, and mergeability harder to preserve correctly.

### Decision: Treat every child Git repository as eligible

The command should attempt the merge in every immediate child Git repository. A missing branch or invalid revision in one repository is a failure for that repository, not a reason to skip it before invoking Git.

Alternative considered: skip repositories where the named local branch does not exist. That would be convenient for partially rolled out branches, but `git merge <commit-ish>` accepts more than local branch names. Pre-filtering would incorrectly reject valid revisions and make the command less Git-like.

### Decision: Best-effort means no cross-repository transaction

If a merge succeeds in one repository and fails in another, the successful merge remains applied. If Git leaves a repository in conflicted merge state, `git-vmr` should leave that state intact and report the failure. Users can then resolve conflicts with normal Git commands inside the affected repositories.

Alternative considered: attempt automatic rollback on any failure. That is risky because a failed merge can leave partially merged files, and automatic cleanup could discard useful conflict state or fail in its own right.

### Decision: Report concise per-repository outcomes

For successful merges, stdout should include the first non-empty line emitted by Git with the repository name appended in parentheses. For failures, stderr should include the first non-empty Git error line with the repository name appended in parentheses. Failures should be sorted by repository name before reporting.

Alternative considered: stream full Git output from each repository as it runs. That preserves all details but becomes noisy and non-deterministic when multiple repositories are involved.

### Decision: Run sequentially first

The initial implementation should run merge attempts sequentially in deterministic repository-name order. Merge is a mutating operation that may prompt users to inspect output carefully, and sequential execution avoids interleaved output and makes tests simpler.

Alternative considered: run merges in parallel like branch creation. That may be faster for many repositories, but merge conflicts and hook behavior are easier to reason about when attempted in a stable order.

## Risks / Trade-offs

- Conflicted merges leave repositories in in-progress merge state -> Document and test this as intentional best-effort behavior.
- The same `<commit-ish>` may resolve differently per repository -> Delegate to Git and report per-repository failures rather than trying to normalize refs across repos.
- Concise reporting may hide useful secondary Git output -> Preserve the first meaningful line for the VMR summary and rely on normal Git commands for detailed follow-up in affected repositories.
- Sequential execution may be slower across many repositories -> Prefer predictable behavior for the first version; parallelism can be revisited later if benchmarks show a need.
