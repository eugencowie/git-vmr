## Context

`git vmr worktree remove` already has two separate safety concepts. The repeated `-f | --force` count is forwarded to `git worktree remove`, while `-d | --delete` records each child worktree's checked-out branch before removal and later deletes that branch with safe `git branch -d` semantics. `git vmr branch` already exposes `-D` as a short-only force-delete shortcut for local branches.

This change adds the same explicit branch force-delete mode to `git vmr worktree remove` without changing the meaning of `--force`.

## Goals / Non-Goals

**Goals:**

- Add short-only `-D` to request forced deletion of branches associated with successfully removed child worktrees.
- Preserve the existing worktree removal force count behavior for `-f | --force`.
- Preserve safe branch deletion for `-d | --delete` and for `--force --delete`.
- Keep branch deletion best-effort and repository-scoped, matching the current `--delete` follow-up phase.
- Reject ambiguous or unsupported CLI shapes before any child repository mutation starts.

**Non-Goals:**

- Add a long `--force-delete` or similar alias for `worktree remove`.
- Reinterpret `-f | --force` as branch force-delete.
- Delete branches for detached child worktrees.
- Delete remote-tracking branches or remote branches.
- Change `worktree add`, `list`, or `move` behavior.

## Decisions

### Represent branch deletion mode explicitly

The remove command should distinguish no branch deletion, safe branch deletion, and forced branch deletion instead of passing only a boolean `delete` flag through the command layer.

Rationale: `-d` and `-D` both request branch deletion, but they differ at the Git branch deletion step. Modeling that state explicitly keeps dispatch readable and avoids inferring force-delete from unrelated worktree force flags.

Alternative considered: add a second boolean such as `force_delete` alongside `delete`. This is enough for a small patch, but the command logic still benefits from collapsing those booleans into a local branch deletion mode before executing removals.

### Keep `-f | --force` scoped to worktree removal

The existing force count continues to be forwarded only to `git worktree remove`. Branch force-delete requires the new `-D` flag.

Rationale: prior behavior intentionally kept `--force --delete` safe for branch deletion. Preserving that contract avoids making an existing command more destructive.

Alternative considered: make `--force --delete` call `git branch -D`. That would be convenient, but it changes the established meaning of `--force` in this subcommand and conflicts with the current spec.

### Use the existing branch deletion helper

After a child worktree removal succeeds and its branch was recorded, the branch deletion phase should call the existing Git helper with `force = true` for `-D` and `force = false` for `-d | --delete`.

Rationale: the helper already maps safe and forced branch deletion to `git branch -d` and `git branch -D`, and it already produces repository-suffixed command results.

Alternative considered: introduce a worktree-specific branch deletion helper. That would duplicate behavior already covered by the branch command path.

### Let CLI parsing reject ambiguous delete modes

`-d | --delete` and `-D` should conflict. Long force-delete forms should remain unsupported by omission from the parser.

Rationale: users get one clear branch deletion mode per invocation, and the requested short-only surface remains enforceable at argument validation.

Alternative considered: let `-D` override `-d` when both are provided. That hides user input mistakes and makes command intent less explicit.

## Risks / Trade-offs

- [Risk] `-D` is visually close to `-d` but more destructive. -> Mitigation: keep it short-only and require an explicit separate flag; do not make `--force --delete` more destructive.
- [Risk] Partial worktree removals can still leave a mix of deleted and retained branches. -> Mitigation: reuse the existing rule that branch deletion runs only after each repository's worktree removal succeeds.
- [Risk] CLI boolean combinations can become harder to reason about. -> Mitigation: parse conflicts at the CLI boundary and convert flags into a small internal branch deletion mode before executing.
