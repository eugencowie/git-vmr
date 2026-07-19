## ADDED Requirements

### Requirement: Combined worktree diff across child repositories
`git vmr diff` SHALL show one combined diff across every child repository in scope, comparing worktree against index by default, concatenating per-repo output byte-faithfully in repository order with no separators. Repositories with an empty diff SHALL contribute nothing.

#### Scenario: Default diff spans all child repos
- **WHEN** `git vmr diff` runs with no arguments in a VMR whose child repos `backend` and `frontend` both have unstaged changes
- **THEN** stdout SHALL contain `backend`'s diff followed by `frontend`'s diff, in repository order, with no separator lines between them

#### Scenario: Clean repos are silent
- **WHEN** `git vmr diff` runs and child repo `tools` has no changes
- **THEN** stdout SHALL contain no output attributable to `tools`

### Requirement: Staged mode
`git vmr diff --staged` SHALL compare index against HEAD instead of worktree against index. `--cached` SHALL be accepted as an alias.

#### Scenario: Staged flag reaches every child
- **WHEN** `git vmr diff --staged` runs
- **THEN** each child invocation SHALL include `--staged`

### Requirement: VMR-root-relative paths via git prefixes
Each child repository SHALL be diffed with `--src-prefix=a/<repo>/` and `--dst-prefix=b/<repo>/` (trailing slashes mandatory), plus `--no-ext-diff` and `--binary`, so all paths in the combined output are VMR-root-relative. Rename detection SHALL remain enabled (no `--no-renames`).

#### Scenario: Prefixed headers
- **WHEN** child repo `backend` has a modified file `src/x`
- **THEN** the combined output SHALL contain `diff --git a/backend/src/x b/backend/src/x`

### Requirement: Rename and copy header rewrite
Because git leaves `rename from`/`rename to`/`copy from`/`copy to` lines unprefixed, each child's output SHALL pass through a pure transform before concatenation that prepends `<repo>/` (without `a/` or `b/`) to the path on those lines. The transform SHALL operate only within extended-header regions — from a `diff --git` line to the first `@@`, or to the next `diff --git` line or end of input when the block has no hunks. For C-quoted path tokens the prefix SHALL be inserted inside the quotes, immediately after the opening `"`, without re-escaping. Coloured header lines SHALL be rewritten identically.

#### Scenario: Rename headers gain the repo prefix
- **WHEN** child repo `backend` renames `old.rs` to `new.rs`
- **THEN** the combined output SHALL contain `rename from backend/old.rs` and `rename to backend/new.rs`

#### Scenario: Hunkless pure rename is rewritten
- **WHEN** a child diff contains a 100% rename block with no `@@` lines
- **THEN** its `rename from`/`rename to` lines SHALL still be rewritten

#### Scenario: Hunk bodies are untouched
- **WHEN** a hunk body contains a line beginning with `rename from`
- **THEN** that line SHALL NOT be rewritten

#### Scenario: Quoted paths keep the prefix inside the quotes
- **WHEN** a rename involves a path that git C-quotes
- **THEN** the repo prefix SHALL appear inside the quotes, after the opening `"`

### Requirement: Applyable patch when piped
When stdout is not a TTY and colour resolves to `never`, the combined output SHALL be a valid patch that `git apply -p1` accepts from the VMR root, covering text, binary, mode-only, symlink, and rename changes. This is a contract; breaking it is a breaking change.

#### Scenario: Apply round-trip
- **WHEN** `git vmr diff` output (captured via a pipe) covering edits and a rename is applied with `git apply -p1` onto a pristine copy of the VMR
- **THEN** the apply SHALL succeed and the resulting trees SHALL match the original working state

### Requirement: Colour gated by one TTY fact
The command SHALL support `--color[=<when>]` with `auto` (default), `always`, and `never`; a bare `--color` means `always`. `auto` SHALL resolve to `always` when stdout is a TTY and `never` otherwise, and the resolved value SHALL be passed explicitly to every child invocation so child configuration cannot inject colour. The same TTY fact SHALL gate colour and paging so they can never disagree.

#### Scenario: Piped output is uncoloured
- **WHEN** `git vmr diff` runs with stdout piped and no `--color` flag
- **THEN** each child invocation SHALL include `--color=never` and stdout SHALL contain no ANSI escape sequences

#### Scenario: Explicit always passes through
- **WHEN** `git vmr diff --color=always` runs with stdout piped
- **THEN** each child invocation SHALL include `--color=always`
- **AND** the applyability contract SHALL NOT apply

### Requirement: Path filters routed to owning repositories
User-supplied paths SHALL be routed to their owning child repositories as repo-relative pathspecs; only owning repositories are diffed. The aggregate path SHALL be allowed and expand to all child repositories. A path owned by no child repository SHALL be an error.

#### Scenario: Path narrows the diff
- **WHEN** `git vmr diff backend/src` runs
- **THEN** only `backend` SHALL be diffed, with pathspec `src`

#### Scenario: Unowned path errors
- **WHEN** `git vmr diff nonexistent/thing` runs and no child repo owns that path
- **THEN** the command SHALL fail with a non-zero exit

### Requirement: Failure aggregation preserves surviving diffs
When a child repository's diff fails, the diffs of succeeding repositories SHALL still print in full on stdout, the failure SHALL report through result aggregation on stderr, and the command SHALL exit non-zero.

#### Scenario: One repo fails
- **WHEN** the diff succeeds in `backend` but git fails in `frontend`
- **THEN** stdout SHALL contain `backend`'s diff, stderr SHALL report the `frontend` failure, and the exit code SHALL be non-zero
