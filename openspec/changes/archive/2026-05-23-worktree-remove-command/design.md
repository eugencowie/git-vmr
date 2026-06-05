## Context

`git vmr worktree add` creates an aggregate linked worktree by resolving one target directory and creating one child Git worktree under it for each immediate child Git repository. It also writes a lightweight `.gitvmr` marker into the aggregate target so existing VMR root discovery works from linked child worktrees.

The inverse operation should fit the same model. A user should be able to provide the aggregate worktree path once, and `git-vmr` should remove the corresponding child worktrees by delegating to Git from each source child repository.

## Goals / Non-Goals

**Goals:**
- Add a nested `worktree remove` command that mirrors Git's command shape.
- Remove aggregate linked worktrees by calling `git worktree remove` once per child Git repository.
- Preserve Git's own validation, dirty-worktree protection, locked-worktree protection, and failure text.
- Support repeated `-f`/`--force` occurrences and pass the same count through to Git for parity with locked worktree removal.
- Keep fan-out behavior best-effort and deterministic.
- Clean up VMR aggregate metadata after fully successful removal.

**Non-Goals:**
- Implement worktree discovery by parsing `git worktree list`.
- Remove arbitrary child worktrees that do not match the aggregate `<worktree>/<repo-name>` layout.
- Transactionally roll back successful removals if another repository fails.
- Add support for other `git worktree` subcommands such as `list`, `move`, `lock`, `prune`, or `repair`.

## Decisions

### Decision: Treat `<worktree>` as an aggregate path

Resolve `<worktree>` relative to the effective working directory, then call Git with `<resolved-worktree>/<repo-name>` for each immediate child Git repository in the source VMR.

Rationale: this is the inverse of `worktree add` and preserves the same aggregate layout. Users operate on the VMR worktree as a single directory while Git still manages each child repository's linked worktree state.

Alternative considered: route `<worktree>` as a normal VMR path. That would point at one child repository path rather than the aggregate root and would not match the command users need to undo `worktree add`.

### Decision: Delegate removal checks to Git

The command should call `git worktree remove` in each child repository and should not pre-check whether the target exists, is clean, is locked, contains submodules, or is the main worktree.

Rationale: Git already owns worktree safety and produces familiar diagnostics. Delegation also keeps behavior aligned across Git versions and avoids duplicating worktree state rules.

Alternative considered: pre-validate target paths and working tree cleanliness before invoking Git. That would make some failures earlier, but it risks diverging from Git and would require parsing worktree metadata.

### Decision: Count and forward force flags

Use clap's counted flag support for `-f`/`--force`, storing the number of occurrences. For each child repository invocation, append `-f` once per occurrence before the worktree path.

Rationale: Git uses repeated force for stronger override behavior, including locked worktree removal. Forwarding the count preserves that behavior without `git-vmr` interpreting the levels itself.

Alternative considered: use a boolean force flag. That would support dirty worktree removal but would block Git's double-force locked-worktree behavior.

### Decision: Preserve best-effort fan-out

Attempt removal for every discovered child Git repository and aggregate successes and failures through the existing repository-suffixed output mechanism. Successful removals should remain removed even if another repository fails.

Rationale: this matches existing VMR fan-out commands and `worktree add`. Removing worktrees is not easily reversible, so rollback would be fragile and could hide the original failure.

Alternative considered: stop at the first failure. That would reduce partial removal, but it would make outcomes depend on repository ordering and force users to rerun repeatedly.

### Decision: Clean aggregate marker only after complete success

After every child removal succeeds, remove `<worktree>/.gitvmr` if present and remove the aggregate directory if it is empty. If any child removal fails, leave the aggregate root and marker untouched.

Rationale: a fully removed aggregate worktree should not remain discoverable as a VMR. During partial failure, leaving the marker helps users run `git vmr` commands from remaining child worktrees while they inspect or retry cleanup.

Alternative considered: always remove `.gitvmr` after any successful child removal. That can strand remaining child worktrees without VMR root discovery after a partial failure.

## Risks / Trade-offs

- [Risk] Partial failures leave an incomplete aggregate worktree. -> Mitigation: report deterministic repository-suffixed failures and leave the marker in place so users can inspect and retry.
- [Risk] Cleanup of the aggregate directory can fail after all Git removals succeed. -> Mitigation: treat cleanup failure as a command failure with a direct filesystem error message; do not attempt to recreate removed child worktrees.
- [Risk] Different Git versions may vary slightly in force semantics or failure text. -> Mitigation: pass force flags through unchanged and assert high-level behavior plus key Git diagnostics in tests.
