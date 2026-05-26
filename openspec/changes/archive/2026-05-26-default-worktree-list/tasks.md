## 1. CLI Behavior

- [x] 1.1 Update worktree CLI parsing/dispatch so an omitted nested worktree subcommand resolves to `WorktreeCommand::List`.
- [x] 1.2 Preserve existing parse failures for unsupported worktree subcommands and invalid `worktree list` operands/options.

## 2. Test Coverage

- [x] 2.1 Add a parser test proving `git-vmr worktree` maps to worktree list behavior.
- [x] 2.2 Add an integration test proving `git vmr worktree` and `git vmr worktree list` produce matching stdout, stderr, and exit status in a VMR fixture.

## 3. Verification

- [x] 3.1 Run the focused CLI/worktree tests.
- [x] 3.2 Run `openspec validate default-worktree-list --strict`.
