## Context

`git vmr worktree remove` currently resolves an aggregate worktree path, attempts `git worktree remove` once per immediate child Git repository, prints repository-suffixed results, and cleans the aggregate `.gitvmr` marker only after every child removal succeeds. The command already delegates dirty, locked, missing, submodule, and main-worktree safety checks to Git.

The branch associated with a child worktree is not always derivable from the aggregate path. `git vmr worktree add ../wt` commonly creates branch `wt`, but callers can use an explicit branch, pass a commit-ish, or create a detached child worktree through Git outside VMR. `worktree remove --delete` therefore needs to inspect each child worktree's actual state before removing it.

## Goals / Non-Goals

**Goals:**

- Add a `-d | --delete` option to remove the checked-out branch after its child worktree is removed.
- Preserve safe branch deletion by using `git branch -d`.
- Preserve best-effort behavior across child repositories.
- Preserve aggregate marker cleanup only when all child worktree removals succeed.
- Report worktree removal and branch deletion failures with deterministic repository suffixes.

**Non-Goals:**

- Add force branch deletion to `git vmr worktree remove`.
- Infer branch names from aggregate path basenames.
- Delete branches for detached child worktrees.
- Delete remote-tracking branches or remote branches.
- Change `git vmr worktree add`, `list`, or `move` behavior.

## Decisions

### Discover branch state before removal

Before removing a child worktree, the command will read that repository's worktree list and find the entry whose path equals `<aggregate-worktree>/<repo-name>`. If the entry is branch-backed, the branch name is recorded for later deletion. If the entry is detached, no branch deletion is scheduled.

Rationale: after `git worktree remove` succeeds, the child worktree path no longer exists, so its checked-out branch may no longer be discoverable from that worktree. Reading the state first also avoids guessing from the aggregate path.

Alternative considered: derive the branch name from the aggregate path basename. This matches the common inferred-branch case but is wrong for explicit branch and detached worktrees.

### Delete only after successful child worktree removal

Branch deletion will run only for repositories whose child worktree removal succeeded and whose pre-removal state was branch-backed. If a child worktree removal fails, its branch will not be deleted.

Rationale: deleting a branch while its worktree still exists is both unsafe and normally rejected by Git. It also preserves the existing best-effort model: successful child removals can continue to follow-up cleanup while failed removals remain intact.

Alternative considered: stop all branch deletion if any child removal fails. That is simpler, but it leaves branches behind for repositories where the worktree was already removed successfully.

### Keep `--force` scoped to worktree removal

The existing repeated `-f | --force` count will continue to be passed only to `git worktree remove`. The new `-d | --delete` flag will use safe `git branch -d` behavior and will not reinterpret `--force` as `git branch -D`.

Rationale: `worktree remove --force --delete` should mean "force worktree removal, then safely delete the branch if Git allows it." Treating the same force flag as force branch deletion would create a surprising destructive path for unmerged branches.

Alternative considered: allow `--force --delete` to use `git branch -D`. This is more powerful, but it overloads an existing worktree safety flag with branch deletion semantics.

### Combine result reporting across phases

The remove command will need to keep enough per-repository outcome information to run the optional branch deletion phase after successful removals, then report failures from both phases. Aggregate marker cleanup will still depend only on complete child worktree removal success, not branch deletion success.

Rationale: branch deletion is a follow-up cleanup step. If every child worktree was removed, the aggregate worktree root should be cleaned even when a branch deletion later fails because the branch is unmerged.

Alternative considered: reuse the current `git::print_results(results)?` call directly after worktree removal. That would bail before branch deletion on partial worktree failures and cannot support the desired per-repository follow-up behavior.

## Risks / Trade-offs

- [Risk] Worktree path matching can be sensitive to path normalization differences between user input and Git porcelain output. -> Mitigation: resolve the aggregate path with the existing worktree remove path logic and compare against the parsed child worktree paths consistently with existing worktree list parsing.
- [Risk] Branch deletion can fail after successful worktree removal, leaving the command with side effects and a non-zero exit. -> Mitigation: document and test this as a best-effort cleanup phase that delegates safe deletion to Git.
- [Risk] Output ordering can become less obvious with two phases. -> Mitigation: keep repository-suffixed grouped messages and deterministic ordering for both worktree removal and branch deletion failures.
