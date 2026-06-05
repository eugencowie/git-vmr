## Context

`git vmr switch <branch-name>` currently discovers the VMR root, scans immediate child Git repositories, and delegates branch switching to `git switch <branch-name>` in each repository. The command runs attempts in parallel, reports deterministic aggregate output, and does not roll back successful repositories after failures.

Git's native `switch` command also supports `-c <new-branch>` and `--create <new-branch>` to create a branch and switch to it. Supporting those forms in `git-vmr` should preserve the existing project pattern: let Git own repository-level semantics while `git-vmr` owns VMR discovery, parallel fan-out, and aggregate reporting.

## Goals / Non-Goals

**Goals:**

- Accept `git vmr switch --create <new-branch>` and `git vmr switch -c <new-branch>`.
- Delegate create-and-switch semantics to Git for each child repository.
- Preserve best-effort fan-out across child repositories.
- Report failures with repository suffixes in deterministic order.
- Deduplicate identical successful Git summary lines so repeated branch creation messages print once.
- Keep existing `git vmr switch <branch-name>` behavior unchanged.

**Non-Goals:**

- Do not support create start points such as `git vmr switch -c feature/auth main`.
- Do not support `--force-create`, `--orphan`, `--track`, `--no-track`, or generic `git switch` option passthroughs.
- Do not pre-check whether branches exist, whether worktrees are clean, or whether create-and-switch will succeed.
- Do not roll back repositories that successfully created and switched after another repository fails.

## Decisions

### Decision: Delegate create-and-switch semantics to Git

The command should invoke `git switch --create <new-branch>` in each child repository when either `-c` or `--create` is provided. This preserves Git behavior for branch naming, branch existence, unborn HEAD handling, worktree safety, hooks, config, and output formatting.

Alternative considered: implement `git vmr switch -c` as `git branch <new-branch>` followed by `git switch <new-branch>`. That would split Git's native porcelain operation into two commands, introduce an extra partial-failure point inside each repository, and risk output or validation differences from user expectations.

### Decision: Support `-c` and `--create`, but not start points

The CLI should accept both Git-compatible create flags because users commonly know the short form from `git switch -c`. The only positional value remains the branch name/new branch name, so extra start-point arguments should fail during CLI parsing.

Alternative considered: support Git's optional `[<start-point>]` argument immediately. That is useful, but it adds another cross-repository resolution surface where the same start point may exist in some repositories and not others. It can be added later with a dedicated requirement.

### Decision: Deduplicate successful create output

For create mode, successful child results should still capture Git's first non-empty summary line, but identical success lines should be printed once without repository suffixes. Failures should continue to use repository suffixes so actionable errors identify the child repository.

Alternative considered: reuse the existing switch output behavior and suffix every success with a repository name. That is consistent with plain branch switching, but `switch -c` commonly produces the same success line in every repository, making the output noisy for large VMRs.

### Decision: Preserve best-effort execution

Create-and-switch attempts should run in every discovered child Git repository even when one or more repositories fail. The command should exit non-zero if any attempt fails and should not roll back repositories that succeeded.

Alternative considered: stop on the first failure or roll back successful repositories after any failure. Both approaches would diverge from the existing mutating command model, and rollback can itself fail or overwrite useful branch state.

## Risks / Trade-offs

- [Risk] Deduplicated success output does not identify every repository that succeeded. -> Mitigation: failures are explicitly repository-suffixed, and users can run `git vmr status` or `git vmr branch` when they need a full inventory.
- [Risk] A branch may be created in some repositories and fail in others. -> Mitigation: keep the behavior explicit in the spec as best-effort with no rollback, matching existing branch mutation semantics.
- [Risk] Future support for start points may need to revisit parsing shape. -> Mitigation: keep this change scoped to a single branch-name positional and reject extra positional arguments now.
