## 1. CLI and Dispatch

- [x] 1.1 Add `force`, `dry_run`, and `cached` fields to the `Rm` CLI command with `-f|--force`, `-n|--dry-run`, and `--cached` parsing.
- [x] 1.2 Update command dispatch and `commands::rm::rm` parameters to carry the new flags alongside `recursive`.
- [x] 1.3 Add CLI parser unit tests for short and long rm flags, including combinations with `-r`.

## 2. Git Invocation Behavior

- [x] 2.1 Update the Git rm wrapper to build `git rm` arguments with `--recursive`, `--force`, `--dry-run`, and `--cached` before `--`.
- [x] 2.2 Preserve quiet success for normal removals while surfacing successful dry-run stdout through repository-suffixed aggregate output.
- [x] 2.3 Keep existing VMR routing and root-recursive validation before invoking any child repository `git rm`.

## 3. Integration Coverage

- [x] 3.1 Add tests for `-f` and `--force` removing modified tracked files.
- [x] 3.2 Add tests for `-n` and `--dry-run` reporting preview output without changing indexes or working trees.
- [x] 3.3 Add tests for `--cached` removing files and directories from the index while leaving working tree files in place.
- [x] 3.4 Add tests that `git vmr rm -n .` still fails without `-r` and `git vmr rm -n -r .` previews aggregate root removal without mutation.
- [x] 3.5 Add tests that `--force`, `--dry-run`, and `--cached` do not bypass VMR path validation.

## 4. Documentation and Verification

- [x] 4.1 Update `docs/git-vmr/rm.md` synopsis and option table to mark `-f|--force`, `-n|--dry-run`, and `--cached` as supported.
- [x] 4.2 Run the focused rm test suite and CLI parser tests.
- [x] 4.3 Run `openspec validate support-rm-command-flags --strict`.
