## 1. CLI Wiring

- [x] 1.1 Add an `Rm` subcommand to the clap command enum with one-or-more path arguments.
- [x] 1.2 Add a `-r` / `--recursive` flag to the `Rm` subcommand.
- [x] 1.3 Add a new `src/cli/rm.rs` module and dispatch `Command::Rm` from `Cli::run`.

## 2. Path Routing

- [x] 2.1 Extract or reuse add-style lexical path normalization without requiring target paths to canonicalize.
- [x] 2.2 Resolve rm arguments relative to the effective working directory from cwd or `-C`.
- [x] 2.3 Validate normalized paths stay inside the VMR root and do not target `.gitvmr/`.
- [x] 2.4 Route child-repo paths to `(repo_path, repo_relative_path)` pairs, using `.` when the path names the repo root.
- [x] 2.5 Expand VMR-root paths such as `.` from the VMR root to `.` for every immediate child Git repository while skipping non-Git directories.
- [x] 2.6 Reject VMR-root expansion before invoking Git unless `-r` / `--recursive` is supplied.
- [x] 2.7 Reject explicit paths owned by non-Git child directories or files directly under the VMR root before removing anything.

## 3. Git Removal

- [x] 3.1 Group routed paths by child repository.
- [x] 3.2 Invoke `git --no-optional-locks -C <repo> rm -- <paths...>` once per repository for non-recursive removals.
- [x] 3.3 Invoke `git --no-optional-locks -C <repo> rm -r -- <paths...>` once per repository when recursive removal is requested.
- [x] 3.4 Surface Git failures with repository context and a non-zero command result.

## 4. Tests

- [x] 4.1 Add integration tests for removing a tracked file from the VMR root.
- [x] 4.2 Add integration tests for removing paths across multiple child repositories in one command.
- [x] 4.3 Add integration tests for removing from inside a child repository and with `-C`.
- [x] 4.4 Add integration tests proving untracked-path removal preserves Git failure semantics and reports repository context.
- [x] 4.5 Add integration tests for recursive directory removal with `-r` and `--recursive`.
- [x] 4.6 Add integration tests proving directory removal without `-r` fails without recursively removing tracked files.
- [x] 4.7 Add integration tests for VMR-root `.` rejection without `-r` and expansion with `-r`.
- [x] 4.8 Add integration tests proving invalid paths fail before any repository is changed.
- [x] 4.9 Add focused unit tests for lexical path normalization and path routing edge cases reused by rm.

## 5. Verification

- [x] 5.1 Run `nix develop -c cargo test`.
- [x] 5.2 Run `nix develop -c openspec status --change rm-command` and confirm the change is apply-ready.
