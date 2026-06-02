## 1. CLI Wiring

- [x] 1.1 Add an `Add` subcommand to the clap command enum with one-or-more path arguments.
- [x] 1.2 Add a new `src/cli/add.rs` module and dispatch `Command::Add` from `Cli::run`.

## 2. Path Routing

- [x] 2.1 Implement lexical path normalization for add arguments without requiring target paths to exist.
- [x] 2.2 Resolve add arguments relative to the effective working directory from cwd or `-C`.
- [x] 2.3 Validate normalized paths stay inside the VMR root and do not target `.gitvmr/`.
- [x] 2.4 Route child-repo paths to `(repo_path, repo_relative_path)` pairs, using `.` when the path names the repo root.
- [x] 2.5 Expand VMR-root paths such as `.` from the VMR root to `.` for every immediate child Git repository while skipping non-Git directories.
- [x] 2.6 Reject explicit paths owned by non-Git child directories or files directly under the VMR root before staging anything.

## 3. Git Staging

- [x] 3.1 Group routed paths by child repository.
- [x] 3.2 Invoke `git --no-optional-locks -C <repo> add -- <paths...>` once per repository.
- [x] 3.3 Surface Git failures with repository context and a non-zero command result.

## 4. Tests

- [x] 4.1 Add integration tests for staging a file from the VMR root.
- [x] 4.2 Add integration tests for staging paths across multiple child repositories in one command.
- [x] 4.3 Add integration tests for staging from inside a child repository and with `-C`.
- [x] 4.4 Add integration tests for `git vmr add .` from the VMR root and from inside a child subtree.
- [x] 4.5 Add integration tests for staging untracked files and deleted files.
- [x] 4.6 Add integration tests proving invalid paths fail before any repository is staged.
- [x] 4.7 Add focused unit tests for lexical path normalization and path routing edge cases.

## 5. Verification

- [x] 5.1 Run `nix develop -c cargo test`.
- [x] 5.2 Run `nix develop -c openspec status --change add-command` and confirm the change is apply-ready.
