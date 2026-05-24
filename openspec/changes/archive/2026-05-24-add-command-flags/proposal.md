## Why

`git vmr add` currently stages explicit routed paths, but common Git workflows also depend on staging all changes, ignored files, and executable-bit changes through `git add` options. Supporting `-A`, `--all`, `-f`, `--force`, and `--chmod` closes that gap while keeping VMR ownership checks in front of child-repository mutation.

## What Changes

- Add `-A` and `--all` support to `git vmr add`, forwarding all-mode staging to child Git repositories.
- Allow `git vmr add -A` or `git vmr add --all` with no pathspecs; this stages `.` in every immediate child Git repository from anywhere inside the VMR.
- Preserve existing path routing when `-A` or `--all` is combined with explicit pathspecs.
- Add `-f` and `--force` support to allow Git to stage otherwise ignored paths after VMR routing succeeds.
- Add `--chmod=+x` and `--chmod=-x` support to update executable bits in child repository indexes.
- Keep existing VMR validation semantics: unsupported ownership, `.gitvmr/`, root files, and paths outside the VMR fail before staging.

## Capabilities

### New Capabilities

### Modified Capabilities
- `add-command`: Extend `git vmr add` behavior for all-mode staging, force staging ignored files, and index executable-bit updates.

## Impact

- Updates the `add` CLI parser shape and command dispatch.
- Extends the Git add wrapper to construct option-aware `git add` invocations.
- Adds parser and integration coverage for all-mode no-path invocation, all-mode scoped path invocation, force staging ignored files, chmod mode changes, validation behavior, and documentation.
- Updates `docs/git-vmr/add.md` supported option table and synopsis.
