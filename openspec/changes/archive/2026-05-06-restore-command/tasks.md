## 1. CLI Surface

- [x] 1.1 Add a new `Restore` subcommand to `src/cli/mod.rs` with required `pathspec` operands.
- [x] 1.2 Add `--staged` and `--worktree` boolean flags for `restore`.
- [x] 1.3 Ensure clap rejects unsupported restore options such as `--source` and extra flag forms outside the scoped command surface.
- [x] 1.4 Add a new `src/cli/restore.rs` module and dispatch `Command::Restore` from `Cli::run`.

## 2. Path Routing

- [x] 2.1 Reuse lexical path resolution so restore operands are interpreted relative to cwd or the global `-C <path>` flag.
- [x] 2.2 Route restore operands to immediate child Git repositories using the existing VMR ownership checks.
- [x] 2.3 Preserve VMR-root expansion so `git vmr restore .` routes `.` to all immediate child Git repositories and skips non-Git child directories.
- [x] 2.4 Validate all restore path ownership before invoking any `git restore` operation.

## 3. Git Restore Execution

- [x] 3.1 Group routed pathspecs by child repository and process repositories in deterministic order.
- [x] 3.2 Invoke `git --no-optional-locks -C <repo> restore -- <paths...>` when no target flags are provided.
- [x] 3.3 Invoke `git restore --staged`, `git restore --worktree`, or `git restore --staged --worktree` according to the selected flags.
- [x] 3.4 Surface Git restore failures with the failing child repository path and Git stderr.

## 4. Integration Tests

- [x] 4.1 Test restoring an unstaged modified tracked file from the VMR root.
- [x] 4.2 Test restoring a deleted tracked file from the VMR root.
- [x] 4.3 Test restoring paths across multiple child repositories.
- [x] 4.4 Test `--staged` unstages a file without discarding working tree modifications.
- [x] 4.5 Test `--worktree` restores working tree changes.
- [x] 4.6 Test combined `--staged --worktree` restores both index and working tree state.
- [x] 4.7 Test restore path interpretation from inside a child repository and with the global `-C <path>` flag.
- [x] 4.8 Test VMR-root expansion for `git vmr restore .` and `git vmr restore --staged .`.
- [x] 4.9 Test invalid ownership failures for paths outside the VMR, `.gitvmr/`, non-Git children, and files directly under the VMR root.
- [x] 4.10 Test untracked file restore failure includes the child repository path.
- [x] 4.11 Test `--source` is rejected without invoking restore behavior.

## 5. Verification

- [x] 5.1 Run `nix develop -c cargo fmt --check`.
- [x] 5.2 Run `nix develop -c cargo test`.
- [x] 5.3 Run `nix develop -c openspec status --change restore-command` and confirm the change is apply-ready.
