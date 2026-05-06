## 1. CLI Wiring

- [x] 1.1 Add a new `Mv` subcommand with exactly `source` and `destination` path operands.
- [x] 1.2 Add a new `src/cli/mv.rs` module and dispatch `Command::Mv` from `Cli::run`.
- [x] 1.3 Ensure clap rejects missing, extra, or unsupported `mv` operands.

## 2. Path Routing

- [x] 2.1 Add or adapt a routing helper that resolves one path to exactly one child Git repository without VMR-root aggregate expansion.
- [x] 2.2 Reuse lexical path resolution relative to cwd or the global `-C <path>` flag for both operands.
- [x] 2.3 Reject paths outside the VMR root, under `.gitvmr/`, directly under the VMR root, in non-Git child directories, or resolving to the VMR root aggregate.
- [x] 2.4 Add focused routing tests for single-owner routing and VMR-root rejection.

## 3. Same-Repository Moves

- [x] 3.1 Detect when source and destination route to the same child repository.
- [x] 3.2 Invoke `git --no-optional-locks -C <repo> mv -- <source> <destination>` for same-repository moves.
- [x] 3.3 Surface same-repository `git mv` failures with the child repository path in the error message.

## 4. Cross-Repository Moves

- [x] 4.1 Detect when source and destination route to different child repositories.
- [x] 4.2 Preflight cross-repository source tracking with Git before moving anything.
- [x] 4.3 Resolve destination-directory semantics, including moves into a child repository root.
- [x] 4.4 Reject cross-repository destination conflicts and missing destination parents before moving.
- [x] 4.5 Move the source filesystem path to the final destination path.
- [x] 4.6 Stage the source deletion with `git add -- <source>` in the source repository.
- [x] 4.7 Stage the destination addition with `git add -- <destination>` in the destination repository.
- [x] 4.8 Surface filesystem and staging failures with useful path or repository context.

## 5. Integration Tests

- [x] 5.1 Add tests for moving a tracked file and tracked directory within one child repository.
- [x] 5.2 Add tests for moving a tracked file and tracked directory between child repositories.
- [x] 5.3 Add tests for moves from inside a child repository and with the global `-C <path>` flag.
- [x] 5.4 Add tests for destination-directory behavior within a repository and into another repository root.
- [x] 5.5 Add tests proving invalid source or destination ownership fails before moving or staging anything.
- [x] 5.6 Add tests proving cross-repository untracked sources fail before moving.
- [x] 5.7 Add tests for same-repository Git failure reporting and cross-repository staging or filesystem failure context where practical.

## 6. Verification

- [x] 6.1 Run `cargo fmt`.
- [x] 6.2 Run `cargo test`.
- [x] 6.3 Run `nix develop -c openspec status --change mv-command` and confirm the change is apply-ready.
