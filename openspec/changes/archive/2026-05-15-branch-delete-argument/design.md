## Context

`git vmr branch` currently has two modes: without a branch name it lists local branches across child repositories, and with a branch name it creates that branch in every discovered child Git repository. Branch creation already establishes the aggregate mutating command pattern for this area: discover repositories once, run Git independently per repository, collect failures, and report each failure with the repository name.

Branch deletion should fit that model while matching Git's own `branch -d` and `branch -D` behavior as closely as possible.

## Goals / Non-Goals

**Goals:**

- Add safe branch deletion with `git vmr branch -d <branch-name>`.
- Add force branch deletion with `git vmr branch -D <branch-name>`.
- Preserve Git's repository-local deletion rules by delegating to `git branch -d` and `git branch -D`.
- Keep deletion best-effort across repositories, with deterministic per-repository failure reports.
- Preserve existing branch listing and branch creation behavior.

**Non-Goals:**

- No deletion of remote-tracking branches.
- No multi-branch deletion in one invocation.
- No rollback of successful deletions when another repository fails.
- No preflight filtering based on merge state, branch existence, or current checkout.

## Decisions

### Decision: Represent branch actions explicitly

The CLI should parse branch invocation into an explicit action: list, create, delete, or force delete. This keeps dispatch clear as the branch subcommand grows beyond the current `Option<String>` shape.

Alternative considered: keep `branch_name: Option<String>` and pass separate booleans through dispatch. That works for the first implementation, but it spreads mode validation and makes mutually exclusive behavior harder to reason about.

### Decision: Delegate deletion semantics to Git

The implementation should shell out to `git branch -d <branch-name>` for safe deletion and `git branch -D <branch-name>` for force deletion in each child repository. Git remains responsible for deciding whether the branch is fully merged, whether the branch exists, and whether the branch is currently checked out.

Alternative considered: pre-check branch presence or merge state before invoking Git. That would duplicate Git behavior, risk divergence from Git error messages, and make edge cases less predictable.

### Decision: Reuse aggregate result reporting

Deletion should use the same aggregate command result path as branch creation. Each repository is attempted independently, success messages are printed with repository suffixes, and failures are collected so the command returns non-zero after all attempts finish.

Alternative considered: stop on the first deletion failure. That would be less useful for VMR workflows because it would leave later repositories untouched even when they could have been handled successfully.

### Decision: Keep delete flags mutually exclusive and require a branch name

`-d` and `-D` should conflict, and either flag should require `<branch-name>`. `git vmr branch -d` without a branch name should fail during CLI validation instead of running a partial or ambiguous operation.

Alternative considered: allow `git vmr branch -d` to mean delete the current branch across repositories. Git does not use that behavior, and deleting the current branch is not generally valid in a repository.

## Risks / Trade-offs

- [Risk] Some repositories may delete the branch while others fail, leaving the VMR in a mixed state. -> Mitigation: make the command explicitly best-effort, report failures with repository names, and rely on `git vmr branch` or a second deletion attempt for follow-up.
- [Risk] Success output may be noisier than branch creation. -> Mitigation: preserve Git-like deletion feedback so users can see which repositories actually deleted the branch.
- [Risk] Parallel execution can scramble output ordering. -> Mitigation: continue using deterministic repository ordering for aggregate result reporting.
