## 1. CLI Parsing

- [x] 1.1 Add `all`, `force`, and `chmod` fields to the `Add` subcommand in `src/commands.rs`.
- [x] 1.2 Allow zero add pathspecs only when `-A` or `--all` is present, while preserving parse failure for bare `git vmr add`.
- [x] 1.3 Add a typed chmod value parser or enum that accepts only `+x` and `-x`.
- [x] 1.4 Add parser unit tests for `-A`, `--all`, no-path all-mode, `-f`, `--force`, `--chmod=+x`, `--chmod=-x`, invalid chmod values, and bare add rejection.

## 2. Command Routing

- [x] 2.1 Update `src/commands/add.rs` to accept add options and pass them through command execution.
- [x] 2.2 For no-path all-mode, expand to `.` in every immediate child Git repository from the discovered VMR root.
- [x] 2.3 Preserve existing routed path behavior when pathspecs are provided with any supported flag.
- [x] 2.4 Preserve existing validation-before-mutation behavior for explicit invalid pathspecs, including when force, all-mode, or chmod is present.

## 3. Git Invocation

- [x] 3.1 Introduce an add options type shared between command routing and `src/git/add.rs`.
- [x] 3.2 Build `git add` arguments with supported flags before `--` and repo-relative paths.
- [x] 3.3 Keep success output quiet and retain existing per-repository failure reporting.

## 4. Integration Tests

- [x] 4.1 Add tests that `git vmr add -A` and `git vmr add --all` stage additions, modifications, and deletions across multiple child repositories with no pathspecs.
- [x] 4.2 Add a test that no-path all-mode works from inside a child repository and still stages every child repository.
- [x] 4.3 Add a test that `git vmr add -A <pathspec>...` preserves routed path scoping.
- [x] 4.4 Add tests that `-f` and `--force` stage ignored files.
- [x] 4.5 Add tests that `--chmod=+x` and `--chmod=-x` update executable bits in the child repository index.
- [x] 4.6 Add tests that invalid path ownership with `--force`, `-A`, and `--chmod` fails before any child repository is staged.

## 5. Documentation and Validation

- [x] 5.1 Update `docs/git-vmr/add.md` synopsis and option table to mark `-A | --all`, `-f | --force`, and `--chmod=(+|-)x` as supported.
- [x] 5.2 Document that no-path `git vmr add -A` and `git vmr add --all` stage all immediate child Git repositories.
- [x] 5.3 Run focused Rust tests for parser and add integration coverage.
- [x] 5.4 Run `nix develop -c openspec validate add-command-flags`.
