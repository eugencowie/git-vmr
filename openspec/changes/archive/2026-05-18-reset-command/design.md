## Context

`git-vmr` operates on immediate child Git repositories under a virtual monorepo root marked by `.gitvmr/`. Existing aggregate commands discover the VMR root from the effective working directory, skip non-Git child directories, shell out to Git for repository-local semantics, and report repository names when rendering aggregate output.

`git vmr reset` should follow the existing fan-out model used by commands such as `merge`, `rebase`, `fetch`, `pull`, and `push`: collect child repositories in deterministic order, run independent Git commands in parallel, buffer results, and report any failures after every repository has been attempted. Reset has destructive modes, so the contract needs to be explicit that this command is best-effort and non-transactional.

## Goals / Non-Goals

**Goals:**

- Add `git vmr reset [--soft | --mixed | --hard | --merge | --keep] [<commit>]`.
- Preserve Git's interpretation of reset modes and commit resolution in each child repository.
- Attempt every discovered child Git repository even when one or more repositories fail.
- Run child repository reset attempts in parallel.
- Report successful reset output and failed reset output with repository suffixes in deterministic repository-name order.
- Reuse existing global `-C <path>` and VMR root discovery behavior.
- Leave each child repository in the state produced by its own `git reset` attempt.

**Non-Goals:**

- Do not support pathspec reset forms such as `git reset [<tree-ish>] -- <pathspec>...`.
- Do not add arbitrary Git reset option passthroughs beyond `--soft`, `--mixed`, `--hard`, `--merge`, and `--keep`.
- Do not pre-resolve commit-ish values or validate working tree/index state before invoking Git.
- Do not recursively discover nested repositories or add configured repository inclusion/exclusion.
- Do not provide cross-repository rollback, transaction semantics, or conflict recovery.
- Do not provide live per-repository progress streaming in the initial implementation.

## Decisions

### Decision: Model reset as an aggregate VMR command

The command should discover the VMR root, collect immediate child Git repositories, and invoke `git --no-optional-locks -C <repo> reset [mode] [<commit>]` in each child repository.

Alternative considered: treat reset like a plain Git passthrough from the effective working directory. That would reset only one repository and would not solve the VMR repetition problem.

### Decision: Keep the initial argument surface narrow

The parser should support exactly one optional reset mode and one optional commit argument. The mode flags are mutually exclusive. When no mode is provided, the command should pass no mode flag and let Git use its default mixed reset behavior. When no commit is provided, the command should pass no commit argument and let Git use its default target.

Alternative considered: collect arbitrary trailing arguments and forward them to Git. That would support more Git syntax immediately, but it would blur the distinction between aggregate reset and path-routed reset, complicate tests, and make destructive behavior harder to document.

### Decision: Exclude pathspec reset for this change

Pathspec reset changes the index for selected paths and should be routed by VMR path ownership, similar to `add`, `restore`, `mv`, and `rm`. This change is scoped to repository-wide reset operations that fan out to every child repository.

Alternative considered: support both repository-wide reset and pathspec reset in one command. That would increase scope substantially because the command would need two execution models: aggregate fan-out for repository resets and lexical path routing for pathspec resets.

### Decision: Delegate reset semantics and validation to Git

The command should not validate whether each child repository can resolve the commit, has a clean working tree, is in a merge state, or can safely apply a requested reset mode. Each child repository may have different refs, local changes, and in-progress Git operations, so Git should decide success or failure per repository.

Alternative considered: preflight repository state before resetting. That could produce friendlier errors for common cases, but it would duplicate Git semantics, risk diverging from Git behavior, and still need to handle races between preflight and reset.

### Decision: Run resets in parallel with buffered reporting

Reset attempts should run in parallel, with each worker capturing Git stdout and stderr. After all workers finish, results should be rendered in deterministic repository-name order.

Alternative considered: stream each child Git process directly to the terminal. That would preserve live output, but parallel output could interleave and become hard to attribute to repositories. Buffered reporting is consistent with existing aggregate command behavior.

### Decision: Best-effort means no cross-repository transaction

If reset succeeds in one repository and fails in another, the successful reset remains applied. The command should return a non-zero status when any repository fails, after attempting every discovered child Git repository.

Alternative considered: stop after the first failure or attempt rollback. Stopping early would leave users without a complete view of which repositories reset successfully. Rollback is not reliable for all reset modes, especially `--hard`, and would be surprising compared with existing best-effort aggregate commands.

## Risks / Trade-offs

- [Risk] `--hard` can discard local work in multiple repositories at once -> Mitigation: make the supported syntax explicit and rely on Git's own reset behavior while documenting that the operation is best-effort and non-transactional.
- [Risk] Best-effort execution can leave repositories at different commits when some resets fail -> Mitigation: report every failed repository after all attempts complete so users can repair only the affected repositories.
- [Risk] Parallel destructive operations can make failures feel less sequentially understandable -> Mitigation: keep aggregate reporting deterministic by repository name.
- [Risk] First-line output can omit details from multi-line Git reports -> Mitigation: keep default output consistent with existing commands and leave detailed investigation to running Git directly in affected repositories.
- [Risk] Excluding pathspec reset may surprise users familiar with full `git reset` -> Mitigation: reject unsupported extra arguments rather than silently forwarding ambiguous pathspec operations.
