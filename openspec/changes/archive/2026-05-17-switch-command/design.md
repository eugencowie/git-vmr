## Context

`git-vmr` operates on immediate child Git repositories under a virtual monorepo root marked by `.gitvmr/`. Existing commands discover the VMR root from the effective working directory, skip non-Git child directories, shell out to Git for repository semantics, and aggregate per-repository results after all attempts complete.

`git vmr branch <branch-name>` and `git vmr branch -d <branch-name>` already establish the best-effort mutating pattern: run independent child repository attempts in parallel, collect every result, print repository-suffixed messages, and return a non-zero status if any repository fails. `git vmr switch <branch-name>` should follow that same model for branch checkout state.

## Goals / Non-Goals

**Goals:**

- Add `git vmr switch <branch-name>` for switching every immediate child Git repository to the same branch name.
- Preserve normal Git switch behavior in each child repository, including dirty-worktree checks, missing branch failures, hooks, and detached-worktree constraints enforced by Git.
- Attempt every discovered child Git repository even when one or more repositories fail.
- Run child repository switch attempts in parallel.
- Report per-repository success and failure output deterministically.
- Reuse the existing global `-C <path>` and VMR root discovery behavior.

**Non-Goals:**

- Do not add branch creation modes such as `-c`, `-C`, or `--orphan` in the initial command.
- Do not add detach mode, discard mode, merge mode, guess mode, or generic `git switch` option passthroughs in the initial command.
- Do not pre-resolve or synchronize branch refs across repositories.
- Do not roll back successful switches if another repository fails.
- Do not support recursive repository discovery or configured repository inclusion/exclusion.

## Decisions

### Decision: Delegate switch semantics directly to Git

The command should shell out to `git --no-optional-locks -C <repo> switch <branch-name>` for each child repository. This keeps behavior aligned with users' Git expectations and avoids reimplementing checkout safety, branch resolution, worktree locking, or dirty-worktree protection in Rust.

Alternative considered: inspect each repository for local branch existence and worktree cleanliness before invoking Git. That could produce friendlier skip messages, but it would duplicate Git behavior and risk diverging from Git's own validation.

### Decision: Support only the branch-name form initially

The first command shape should be `git vmr switch <branch-name>`. Branch creation and advanced checkout modes should remain out of scope until there is a clear requirement for how those flags should behave across repositories.

Alternative considered: mirror a broader subset of `git switch` flags immediately. That would make the command more complete, but it increases CLI validation complexity and creates ambiguous cross-repository behavior for branch creation and discard operations.

### Decision: Run switches in parallel

The command should run child repository switch attempts in parallel and capture each repository's output before rendering aggregate results. Child repositories are independent working trees, and parallel execution matches existing branch mutation behavior while keeping large VMRs responsive.

Alternative considered: run switches sequentially. Sequential execution is simpler, but there is no dependency between child repositories and output can remain deterministic by collecting results before rendering.

### Decision: Best-effort means no cross-repository transaction

If switching succeeds in one repository and fails in another, the successful repository remains switched. The command should not attempt to remember previous branches or restore repositories after a later failure.

Alternative considered: roll back repositories that switched successfully after any failure. That would be fragile because previous HEAD state can be detached, branches may move, and a rollback can itself fail or overwrite useful user state.

### Decision: Report repository-suffixed outcomes after all attempts

Successful switches should print Git's first non-empty success line with the repository name appended in parentheses. Failed switches should report the first non-empty Git stderr line, falling back to stdout if needed, with the repository name appended in parentheses. Rendering should happen after all parallel attempts complete so failures are aggregated at the end and reported in deterministic repository order.

Alternative considered: stream raw Git output from each worker. That would preserve full Git output but would interleave under parallel execution and lose the consistent repository suffixes used by other `git-vmr` commands.

## Risks / Trade-offs

- [Risk] Some repositories may switch while others fail, leaving the VMR on mixed branches -> Mitigation: specify best-effort behavior, report failed repositories, and rely on a follow-up `git vmr switch` or `git vmr branch`/`git vmr status` for inspection.
- [Risk] The same branch name may resolve differently per repository -> Mitigation: delegate to Git and report per-repository failures rather than trying to normalize refs across repositories.
- [Risk] Printing successful switch messages can be noisier than quiet commands such as `rebase` -> Mitigation: use concise first-line Git output with repository suffixes so users can see which repositories actually switched.
- [Risk] Parallel execution can make raw Git output unreadable -> Mitigation: capture per-repository output and render deterministic aggregate messages after all attempts finish.
