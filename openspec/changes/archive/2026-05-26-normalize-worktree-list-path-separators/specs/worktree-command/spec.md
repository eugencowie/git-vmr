## ADDED Requirements

### Requirement: Worktree list renders Git-style path separators
The `git vmr worktree list` command SHALL render aggregate worktree paths with Git-style `/` separators, regardless of the separator style used by the underlying platform path display or Git worktree list input.

#### Scenario: Windows-style aggregate path is normalized
- **WHEN** `git vmr worktree list` renders an aggregate worktree path represented as `C:\Projects\vmr`
- **THEN** the output SHALL contain `C:/Projects/vmr`
- **AND** the output SHALL NOT contain `C:\Projects\vmr`

#### Scenario: Mixed path sources render consistently
- **WHEN** `git vmr worktree list` renders aggregate worktree paths represented as `C:\Projects\vmr` and `C:/Worktrees/new-feature`
- **THEN** the output SHALL contain `C:/Projects/vmr`
- **AND** the output SHALL contain `C:/Worktrees/new-feature`
- **AND** both rendered paths SHALL use `/` as the path separator
