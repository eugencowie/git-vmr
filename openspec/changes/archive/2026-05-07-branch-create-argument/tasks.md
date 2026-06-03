## 1. CLI Shape

- [x] 1.1 Update the `Branch` CLI variant to accept an optional `[branch-name]` positional argument.
- [x] 1.2 Add CLI parser tests covering `git-vmr branch` list mode and `git-vmr branch <branch-name>` create mode.
- [x] 1.3 Dispatch the optional branch name from `Cli::run` into the branch command module.

## 2. Branch Creation Implementation

- [x] 2.1 Split branch command behavior so no argument preserves current list-mode output.
- [x] 2.2 Add child repository discovery for create mode that skips non-Git child directories and keeps repository names available for reporting.
- [x] 2.3 Run `git --no-optional-locks -C <repo> branch <branch-name>` independently for each discovered child Git repository.
- [x] 2.4 Execute branch creation attempts in parallel while capturing each repository result instead of writing directly to stdout or stderr.
- [x] 2.5 Sort failed repository results by repository name and render one stderr line per failure using the first non-empty Git stderr line followed by ` (<repo-name>)`.
- [x] 2.6 Return success with no output when every repository creates the branch successfully, and return a non-zero error after reporting when any repository fails.

## 3. Tests

- [x] 3.1 Add an integration test that `git vmr branch <branch-name>` creates the branch in every child Git repository and prints no output on success.
- [x] 3.2 Add an integration test that non-Git child directories are skipped during branch creation.
- [x] 3.3 Add an integration test where one repository fails but other repositories still create the branch.
- [x] 3.4 Add an integration test for concise failure reporting with repository suffixes.
- [x] 3.5 Add a test that multiple failure lines are reported in deterministic repository-name order.
- [x] 3.6 Run the existing branch listing tests to verify no-argument behavior is unchanged.

## 4. Validation

- [x] 4.1 Run `nix develop -c cargo fmt --check`.
- [x] 4.2 Run `nix develop -c cargo test`.
- [x] 4.3 Run `nix develop -c openspec validate branch-create-argument --strict`.
