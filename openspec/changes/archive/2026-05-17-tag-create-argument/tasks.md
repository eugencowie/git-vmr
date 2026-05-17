## 1. CLI Shape

- [x] 1.1 Update the `Tag` CLI variant to accept an optional `[tag-name]` positional argument.
- [x] 1.2 Add CLI parser tests covering `git-vmr tag` list mode and `git-vmr tag <tag-name>` create mode.
- [x] 1.3 Add CLI parser tests proving unsupported extra operands and tag flags are rejected.
- [x] 1.4 Dispatch the optional tag name from `Cli::run` into the tag command module.

## 2. Tag Creation Implementation

- [x] 2.1 Split tag command behavior so no argument preserves current list-mode output.
- [x] 2.2 Add create mode that discovers immediate child Git repositories and skips non-Git child directories.
- [x] 2.3 Add Git wrapper support for `git --no-optional-locks -C <repo> tag <tag-name>`.
- [x] 2.4 Execute tag creation attempts in parallel while capturing each repository result instead of writing directly to stdout or stderr.
- [x] 2.5 Sort failed repository results by repository name and render one stderr line per failure using the first non-empty Git stderr line followed by ` (<repo-name>)`.
- [x] 2.6 Return success with no output when every repository creates the tag successfully, and return a non-zero error after reporting when any repository fails.

## 3. Tests

- [x] 3.1 Add an integration test that `git vmr tag <tag-name>` creates the lightweight tag in every child Git repository and prints no output on success.
- [x] 3.2 Add an integration test that non-Git child directories are skipped during tag creation.
- [x] 3.3 Add an integration test where one repository fails but other repositories still create the tag.
- [x] 3.4 Add an integration test for concise failure reporting with repository suffixes.
- [x] 3.5 Add a test that multiple failure lines are reported in deterministic repository-name order.
- [x] 3.6 Run existing tag listing tests to verify no-argument behavior is unchanged.

## 4. Documentation

- [x] 4.1 Update `docs/git-vmr/tag.md` to document lightweight tag creation support and unsupported tag options.
- [x] 4.2 Update summary documentation if needed so `tag` is no longer described as list-only.

## 5. Validation

- [x] 5.1 Run `nix develop -c cargo fmt --check`.
- [x] 5.2 Run `nix develop -c cargo test`.
- [x] 5.3 Run `nix develop -c openspec validate tag-create-argument --strict`.
