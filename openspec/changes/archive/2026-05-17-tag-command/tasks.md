## 1. CLI Wiring

- [x] 1.1 Add a `Tag` subcommand to the clap command enum with Git-compatible list-only help text.
- [x] 1.2 Add a new `src/cli/tag.rs` module and dispatch `Command::Tag` from `Cli::run`.
- [x] 1.3 Ensure unsupported tag operands and flags fail during CLI argument validation.

## 2. Tag Collection

- [x] 2.1 Discover the VMR root from the effective working directory using the existing VMR root discovery function.
- [x] 2.2 Scan immediate child directories, skip non-Git children, and keep repository ordering deterministic.
- [x] 2.3 Add Git wrapper support for collecting local tag refs from each child Git repository with `git for-each-ref --format=%(refname:short) refs/tags`.
- [x] 2.4 Return an error when a child that appears to be a Git repository cannot provide tag information.

## 3. Rendering

- [x] 3.1 Group collected tag refs by tag name across discovered child repositories.
- [x] 3.2 Render tag lines in deterministic tag-name order.
- [x] 3.3 Omit repository names for tags that exist in every discovered child Git repository.
- [x] 3.4 Include sorted repository names for tags that exist in only a subset of discovered child Git repositories.
- [x] 3.5 Ensure no child repositories or no local tag refs produces successful empty output.

## 4. Documentation

- [x] 4.1 Add `docs/git-vmr/tag.md` documenting the list-only supported command surface.
- [x] 4.2 Update `README.md` and `docs/git-vmr.md` to mark `tag` as supported and link to the new command documentation.

## 5. Tests

- [x] 5.1 Add rendering unit tests for shared tags, partial tags, sorted repository lists, and empty output.
- [x] 5.2 Add integration tests for `git vmr tag` across multiple child repositories with local-only tag inventory.
- [x] 5.3 Add integration tests for non-Git child skipping, no-child-repository output, and no-local-tag output.
- [x] 5.4 Add integration tests for nested working directory discovery and global `-C <path>` behavior.
- [x] 5.5 Add an integration test for corrupted child Git repository failure.
- [x] 5.6 Add CLI parsing tests proving creation, deletion, filtering, and other unsupported tag arguments are rejected.

## 6. Verification

- [x] 6.1 Run `nix develop -c cargo fmt --check`.
- [x] 6.2 Run `nix develop -c cargo test`.
- [x] 6.3 Run `nix develop -c openspec validate tag-command --strict`.
