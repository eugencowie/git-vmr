## Why

`git vmr switch <branch>` can emit many near-identical success lines when child repositories are behind the same upstream by different commit counts. The current aggregate renderer groups exact messages, so the variable `by N commits` phrase prevents otherwise identical switch output from merging.

## What Changes

- Normalize successful `git switch` behind-and-fast-forward advisory lines before aggregate rendering.
- Remove only the variable `by N commit` or `by N commits` phrase from messages shaped like `Your branch is behind '<upstream>' by N commits, and can be fast-forwarded.`
- Preserve the upstream branch name, fast-forward wording, repository attribution, exit behavior, and all other switch output.
- Do not add fuzzy matching or broader aggregate-output normalization.
- Add regression coverage for multiple repositories behind the same upstream by different commit counts rendering as one grouped message.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `switch-command`: Successful branch-switch output normalizes Git's variable behind-count phrase so equivalent behind-and-fast-forward advisories can be grouped.

## Impact

- Affected code: `src/git/switch.rs`
- Affected tests: `tests/switch.rs`
- Affected specs: `openspec/specs/switch-command/spec.md`
- No new dependencies, CLI flags, or breaking changes.
