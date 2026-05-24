## Why

`git vmr rm` currently handles routed tracked-file removal across child repositories, but it omits common `git rm` safety and index-management flags. Supporting force, dry-run, and cached removal closes practical workflow gaps while preserving VMR ownership validation before any child repository is touched.

## What Changes

- Add `-f` and `--force` support to `git vmr rm`, forwarding force removal to each affected child repository after VMR routing succeeds.
- Add `-n` and `--dry-run` support so users can preview routed removals without modifying child repository indexes or working trees.
- Add `--cached` support so users can remove paths from child repository indexes while leaving working tree files in place.
- Preserve existing `-r` behavior, including the requirement that VMR-root expansion still needs explicit recursive intent.
- Keep existing VMR validation semantics: paths outside the VMR, paths under `.gitvmr/`, files directly under the VMR root, and explicit non-Git child paths fail before any `git rm` invocation, even with `--force`, `--dry-run`, or `--cached`.

## Capabilities

### New Capabilities

### Modified Capabilities
- `rm-command`: Extend `git vmr rm` behavior for force removal, dry-run previews, and cached index-only removal.

## Impact

- Updates the `rm` CLI parser shape and command dispatch.
- Extends the Git rm wrapper to construct option-aware `git rm` invocations.
- Adds parser and integration coverage for force, dry-run, cached removal, VMR-root dry-run behavior, and validation behavior with flags.
- Updates `docs/git-vmr/rm.md` supported option table and synopsis.
