## 1. Result Model

- [x] 1.1 Replace the fan-out Git command result alias with a structured outcome that preserves repository name, status, and raw Git message separately.
- [x] 1.2 Add helper constructors for successful quiet outcomes, successful message outcomes, and failed message outcomes.
- [x] 1.3 Update the shared Git output helper so adapters can produce structured outcomes without appending repository suffixes.

## 2. Aggregate Renderer

- [x] 2.1 Update CLI result rendering to group successful outcomes by exact message and render each message once.
- [x] 2.2 Update CLI error rendering to group failed outcomes by exact message and render each message once.
- [x] 2.3 Add grouped repository suffix formatting for single-repo, small multi-repo, and large multi-repo groups.
- [x] 2.4 Ensure failure rendering emits only one `fatal:` prefix when Git output already starts with `fatal:`.
- [x] 2.5 Preserve quiet output when all successful child operations emit no Git message.

## 3. Command Migration

- [x] 3.1 Migrate branch, switch, tag, commit, merge, fetch, pull, push, reset, and rebase adapters to return structured outcomes.
- [x] 3.2 Migrate add, restore, and rm adapters to return structured outcomes for quiet successes and failures.
- [x] 3.3 Remove the switch-create-only success deduplication helper after shared rendering covers the behavior.
- [x] 3.4 Keep non-fan-out command behavior unchanged for clone, init, and cross-repository mv.

## 4. Tests

- [x] 4.1 Add unit tests for grouped success rendering with one repository, multiple named repositories, and large repository counts.
- [x] 4.2 Add unit tests for grouped failure rendering and duplicate fatal-prefix prevention.
- [x] 4.3 Update command integration tests that currently expect one line per repository to expect grouped output.
- [x] 4.4 Add integration coverage for mixed grouped successes and grouped failures in the same command invocation.
- [x] 4.5 Add regression coverage that quiet successes do not produce output and do not hide failures.

## 5. Verification

- [x] 5.1 Run the Rust test suite.
- [x] 5.2 Run `openspec validate aggregate-git-output-messages --strict`.
