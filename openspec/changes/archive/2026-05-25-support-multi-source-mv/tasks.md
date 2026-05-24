## 1. CLI Parsing

- [x] 1.1 Change the `mv` command parser to accept two or more path operands.
- [x] 1.2 Split parsed operands into one or more sources plus one destination in the command runner.
- [x] 1.3 Update CLI unit tests to accept multi-source `mv` operands and continue rejecting zero, one, and unsupported option operands.

## 2. Move Planning

- [x] 2.1 Introduce a move plan representation that records each routed source, its final routed destination, and whether the command is using the multi-source form.
- [x] 2.2 Preserve existing two-operand destination handling, including rename-to-path and existing-directory semantics.
- [x] 2.3 Require the destination operand to be an existing directory when more than one source is provided.
- [x] 2.4 Preflight every source and final destination before mutation, including invalid ownership, untracked sources, existing final destinations, missing source basenames, and duplicate final destinations.

## 3. Move Execution

- [x] 3.1 Extend the Git move wrapper to invoke `git mv -- <source>... <destination-directory>` for all-same-repository multi-source plans.
- [x] 3.2 Execute cross-repository or mixed-repository plans sequentially with filesystem rename, source deletion staging, and destination addition staging.
- [x] 3.3 Keep error messages scoped to the failing repository or filesystem path and preserve existing same-repository Git failure reporting.

## 4. Tests

- [x] 4.1 Add integration tests for moving multiple files within one child repository.
- [x] 4.2 Add integration tests for moving multiple files from one repository into another repository root.
- [x] 4.3 Add integration tests for moving sources from multiple repositories into one destination directory.
- [x] 4.4 Add integration tests for multi-source moves from a child working directory and through the global `-C` option.
- [x] 4.5 Add preflight failure tests for non-directory destination, existing final destination, duplicate final destination, invalid source ownership, and untracked source.
- [x] 4.6 Run the relevant test suite and `openspec validate support-multi-source-mv --strict`.

## 5. Documentation

- [x] 5.1 Update `docs/git-vmr/mv.md` to mark the multi-source destination-directory synopsis as supported.
- [x] 5.2 Update any command summary or generated documentation source that reflects the `mv` operand shape.
