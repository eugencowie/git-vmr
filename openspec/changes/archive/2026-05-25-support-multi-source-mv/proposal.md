## Why

`git vmr mv` currently supports only the two-operand rename form, while Git also supports moving multiple sources into an existing destination directory. Supporting the second form lets users move batches of tracked paths across the virtual monorepo without dropping to individual child repositories or composing manual filesystem and staging commands.

## What Changes

- Accept `git vmr mv <source>... <destination-directory>` in addition to the existing `git vmr mv <source> <destination>` form.
- Preserve existing two-operand rename and destination-directory behavior.
- Treat the final operand as the destination directory when more than one source is provided.
- Move each source under the destination directory using its basename, including moves whose sources span multiple child repositories.
- Validate all sources, destination ownership, destination directory shape, destination conflicts, and duplicate final paths before moving any source.
- Continue to reject unsupported `git mv` flags such as `--dry-run`, `-k`, and `--force`.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `mv-command`: Add the multi-source destination-directory form and its validation, routing, and staging behavior.

## Impact

- Updates CLI parsing for `mv` operands.
- Extends `src/commands/mv.rs` and `src/git/mv.rs` to support multi-source moves.
- Adds integration coverage for same-repository and cross-repository multi-source moves, destination-directory validation, duplicate final path rejection, and preflight no-move guarantees.
- Updates `git vmr mv` command documentation.
