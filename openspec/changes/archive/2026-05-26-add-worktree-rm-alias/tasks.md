## 1. CLI Alias

- [x] 1.1 Add a visible `rm` alias to the `worktree remove` Clap subcommand without creating a separate command variant.
- [x] 1.2 Verify `git vmr worktree --help` advertises `rm` as an alias for `remove`.

## 2. Tests

- [x] 2.1 Add parser coverage proving `git vmr worktree rm ../wt` maps to `WorktreeCommand::Remove`.
- [x] 2.2 Add parser coverage proving representative remove flags, such as `--force -D`, parse identically through the `rm` alias.
- [x] 2.3 Add integration coverage proving `git vmr worktree rm ../wt` removes aggregate child worktrees through the existing remove path.

## 3. Verification

- [x] 3.1 Run the focused CLI parser tests.
- [x] 3.2 Run the focused worktree integration tests.
- [x] 3.3 Run OpenSpec validation for `add-worktree-rm-alias`.
