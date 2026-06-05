## 1. Message Capture

- [x] 1.1 Replace branch and tag command uses of fatal-stripping helpers with non-stripping first-line helpers.
- [x] 1.2 Replace rebase and reset failure message selection with non-stripping stderr/stdout fallback helpers.
- [x] 1.3 Remove unused fatal-stripping helper functions from the Git adapter module.

## 2. Aggregate Rendering

- [x] 2.1 Print aggregate error messages exactly as rendered by grouping, without routing them through `fatal:` prefix formatting.
- [x] 2.2 Remove the aggregate `fatal_message` helper and its unit tests from the top-level entry point.
- [x] 2.3 Keep non-aggregate top-level error rendering unchanged.

## 3. Tests And Specs

- [x] 3.1 Update branch deletion failure tests to expect Git's `error:` prefix without a synthetic `fatal:`.
- [x] 3.2 Update branch, tag, rebase, and reset tests affected by removed fatal-stripping helpers while preserving expectations for Git messages that already include `fatal:`.
- [x] 3.3 Add or update aggregate rendering coverage for a Git failure message that begins with `error:` and must not be prefixed with `fatal:`.
- [x] 3.4 Run the relevant Rust test suite and OpenSpec validation for `preserve-git-failure-messages`.
