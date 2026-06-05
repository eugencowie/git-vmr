## Why

Virtual monorepos can already coordinate branch and checkout operations across child repositories, but there is no VMR-level way to create a matching linked worktree for every child repository. Users need `git vmr worktree add <path> [<commit-ish>]` so a new aggregate worktree has the same repository layout and can be used from any child directory.

## What Changes

- Add a `git vmr worktree add <path> [<commit-ish>]` command.
- When `<commit-ish>` is omitted, derive one branch name from the aggregate target path basename and create each child worktree on that same new branch.
- When `<commit-ish>` is provided, pass it directly to Git worktree add in each child repository and let Git resolve or reject it.
- Create a `.gitvmr` marker in the linked aggregate worktree so existing VMR root discovery works from the new worktree.
- Use best-effort execution across child repositories: attempt every child repository and leave successful worktrees in place if other repositories fail.
- Preserve deterministic aggregate output and repository-suffixed failure reporting consistent with existing fan-out commands.

## Capabilities

### New Capabilities
- `worktree-command`: Defines VMR-level worktree management, starting with `worktree add`.

### Modified Capabilities

## Impact

- CLI parsing gains a nested `worktree add` command shape.
- Command dispatch gains a worktree command module.
- Git delegation gains `git worktree add` helpers for branch-inferred and explicit commit-ish modes.
- VMR marker creation is needed for linked aggregate worktree roots.
- Integration tests should cover branch inference, explicit commit-ish errors, best-effort behavior, nested discovery from linked worktrees, and `-C` path handling.
