## Why

`git-vmr` currently applies `fatal:` to every non-aggregate top-level error in the entry point, while aggregate Git failures preserve Git's selected message text. This makes tool-authored error prefixes a global formatting side effect instead of an explicit choice, and prevents `git-vmr` from matching Git's mixed `fatal:`, `error:`, command-prefixed, and unprefixed error style on a per-failure basis.

## What Changes

- Introduce an explicit CLI error rendering contract for `git-vmr` authored errors.
- Replace top-level automatic `fatal:` prefixing with verbatim rendering of already-rendered CLI errors.
- Classify direct non-aggregate errors at their creation or command boundary as `fatal:`, `error:`, or intentionally unprefixed.
- Keep fan-out Git failure messages verbatim, including any prefix Git emitted.
- Keep non-zero exit behavior unchanged for both direct errors and aggregate failures.
- Remove or simplify the aggregate error special case once aggregate failures can be represented as render-ready error text without top-level prefixing.

## Capabilities

### New Capabilities

- `cli-error-rendering`: User-facing stderr rendering for direct `git-vmr` errors, including explicit prefix selection and verbatim top-level output.

### Modified Capabilities

None.

## Impact

- Affects `src/main.rs` top-level error rendering.
- Affects direct error creation and propagation in CLI command modules, VMR routing/discovery, and Git helper paths that reach the top level outside aggregate rendering.
- Affects tests that assert `fatal:` for tool-authored non-aggregate errors.
- No CLI argument, dependency, or exit-code changes.
