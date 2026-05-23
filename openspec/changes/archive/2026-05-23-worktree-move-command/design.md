## Context

`git vmr worktree add` creates an aggregate linked worktree by resolving one target directory, creating a child Git worktree under that target for each immediate child Git repository, and writing a `.gitvmr` marker at the aggregate root. `git vmr worktree remove` performs the inverse operation by resolving the aggregate worktree path, delegating removal to Git for each child repository, and cleaning the aggregate marker only after complete success.

`git vmr worktree move` should fit the same aggregate model. A user should provide the source aggregate worktree and destination aggregate path once, and `git-vmr` should move the corresponding child linked worktrees by delegating to Git from each source child repository.

## Goals / Non-Goals

**Goals:**

- Add a nested `worktree move` command that mirrors Git's command shape.
- Move aggregate linked worktrees by calling `git worktree move` once per child Git repository.
- Preserve Git's own validation, dirty-worktree protection, locked-worktree protection, submodule checks, and failure text.
- Support `-f` and `--force` by forwarding the requested force occurrences to every child Git invocation.
- Keep VMR root discovery working from moved child worktrees by maintaining aggregate `.gitvmr` markers.

**Non-Goals:**

- Implement worktree discovery by parsing `git worktree list`.
- Move arbitrary child worktrees that do not match the aggregate `<worktree>/<repo-name>` layout.
- Add support for other `git worktree` subcommands such as `list`, `lock`, `prune`, `repair`, or `unlock`.
- Add `--relative-paths` support.
- Roll back successful child moves when another child move fails.

## Decisions

### Decision: Treat both operands as aggregate paths

Resolve `<worktree>` and `<new-path>` relative to the effective working directory. For each immediate child Git repository under the source VMR, call Git with `<resolved-worktree>/<repo-name>` and `<resolved-new-path>/<repo-name>`.

Rationale: this matches `worktree add` and `worktree remove`. Users operate on the VMR worktree as a single aggregate directory while Git still manages each child repository's linked worktree metadata.

Alternative considered: route operands as normal VMR paths. That would point at individual child repository paths rather than aggregate roots and would not match the command users need when moving a linked aggregate worktree created by `git vmr worktree add`.

### Decision: Delegate child worktree moves to Git

The command should call `git worktree move` in each child repository and should not pre-check whether the source exists, destination exists, worktree is dirty, worktree is locked, contains submodules, or is a main worktree.

Rationale: Git owns worktree safety rules and diagnostics. Delegating keeps behavior aligned across Git versions and avoids duplicating worktree state rules in `git-vmr`.

Alternative considered: pre-validate source and destination paths before invoking Git. That could make some failures earlier, but it risks diverging from Git and would require parsing or reconstructing Git's worktree state.

### Decision: Create the destination aggregate root before fan-out

Create `<new-path>` and write `<new-path>/.gitvmr` before invoking child `git worktree move` commands.

Rationale: Git can move a child worktree to `<new-path>/<repo-name>`, but the aggregate destination parent must exist. `git-vmr` already owns aggregate directory and marker setup for `worktree add`, so it should own the same setup for aggregate moves.

Alternative considered: let the first Git invocation create any missing parent directories. Git does not create missing destination parents for nested paths, so this would fail for the common aggregate move case.

### Decision: Keep markers after partial or total move failure

After every child move succeeds, remove `<worktree>/.gitvmr` if present and remove the old aggregate directory if it is empty. If any child move fails, leave both the old marker and the destination marker in place, including when all child moves fail after destination setup.

Rationale: partial moves can leave valid child worktrees under both aggregate roots. Keeping markers preserves VMR root discovery while users inspect or retry. Leaving the destination marker after total failure matches the existing `worktree add` best-effort behavior, where aggregate setup is not rolled back after child failures.

Alternative considered: remove the destination marker when no child move succeeds. That would reduce empty marker directories, but it would introduce rollback behavior not present in `worktree add` and require distinguishing total failure from partial failure after parallel execution.

### Decision: Count and forward force occurrences

Use clap's counted flag support for `-f` and `--force`, storing the number of occurrences. For each child repository invocation, append `-f` once per occurrence before the worktree operands.

Rationale: the existing `worktree remove` command uses counted force flags. Reusing the same model keeps CLI behavior consistent and delegates the meaning of force levels to Git.

Alternative considered: use a boolean force flag. That would support common forced moves but would diverge from the existing counted-force implementation and prevent exact forwarding of repeated force flags.

### Decision: Best-effort execution without rollback

Execute child moves for every discovered child Git repository and report deterministic repository-suffixed results. Do not move successful children back if another child fails.

Rationale: this matches existing VMR fan-out commands and `worktree add/remove`. Rollback would require another set of Git worktree moves that can independently fail, potentially hiding the original error and making the final state less obvious.

Alternative considered: stop on first failure. That would avoid partial aggregate moves but would make behavior dependent on repository ordering and diverge from existing worktree fan-out behavior.

## Risks / Trade-offs

- [Risk] Partial failures can split an aggregate worktree across old and new aggregate roots. -> Mitigation: report deterministic repository-suffixed failures and keep both markers so users can inspect or retry from either side.
- [Risk] Destination aggregate setup can leave an empty `.gitvmr` marker if every child move fails. -> Mitigation: document and test this as intentional best-effort behavior consistent with `worktree add`.
- [Risk] Cleanup of the old aggregate directory can fail after all child moves succeed. -> Mitigation: treat cleanup failure as a command failure with a direct filesystem error message; do not attempt to move child worktrees back.
