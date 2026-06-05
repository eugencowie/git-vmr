## Why

Fan-out Git command failures currently receive special handling for the `fatal:` prefix: some adapters strip it from Git output and the top-level renderer conditionally adds it back. This makes the output less faithful to Git and creates command-specific behavior that users have to reason about.

## What Changes

- Preserve the selected Git output line as Git emitted it, including any `fatal:`, `error:`, or other prefix.
- Remove aggregate failure rendering logic that conditionally adds `fatal:` to grouped Git failure messages.
- Remove Git adapter helpers and command paths that strip `fatal: ` from captured Git output.
- Keep repository suffixing, grouping, quiet-success behavior, and non-zero aggregate exit semantics unchanged.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `aggregate-command-output`: Fan-out failure rendering will preserve the selected Git message verbatim instead of normalizing or adding the `fatal:` prefix.
- `branch-command`: Branch deletion failure examples will preserve Git's `error:` output without adding a synthetic `fatal:` prefix.

## Impact

- Affects aggregate failure rendering in `src/main.rs` and fan-out Git message capture helpers in `src/git.rs`.
- Affects commands currently using fatal-stripping helpers, including branch, tag, rebase, and reset.
- Requires updating tests and specs that expect synthetic `fatal:` prefixes for Git messages that do not include them.
- No dependency or CLI argument changes.
