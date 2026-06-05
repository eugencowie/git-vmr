## ADDED Requirements

### Requirement: Bare worktree command defaults to list
The `git vmr worktree` command with no nested subcommand SHALL behave the same as `git vmr worktree list`.

#### Scenario: Bare worktree command lists aggregate worktrees
- **WHEN** user runs `git vmr worktree`
- **THEN** the command SHALL perform the same worktree listing behavior as `git vmr worktree list`
- **AND** stdout, stderr, and exit status SHALL match `git vmr worktree list`
