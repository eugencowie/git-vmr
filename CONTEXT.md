# git-vmr

A Git CLI wrapper for treating multiple independent repositories as a single unified workspace — the flexibility of independent repositories with the ergonomics of a monorepo.

## Language

### Workspace

**VMR**:
A virtual monorepo — a directory on disk that groups independent Git repositories, identified by a `.gitvmr` marker.
_Avoid_: monorepo, root repo

**Workspace**:
A VMR opened by one command: the child repos discovered at that moment, held fixed for the duration of the command, plus the means to run git across them. The disk can change mid-command; the workspace cannot.
_Avoid_: VMR (when the fixed, opened view is meant), context, session

**Child repo**:
A Git repository living directly inside a VMR, operated on as part of the whole.
_Avoid_: submodule, subrepo, member, project

**VMR root**:
The directory bearing the `.gitvmr` marker: the main root of a VMR, or a worktree root.
_Avoid_: boundary, top level

**Working dir**:
The directory a command is invoked from, or the override given with `-C`. Determines how user-supplied paths are interpreted.
_Avoid_: cwd, current directory

### Paths

**Routing**:
Deciding which child repo owns a user-supplied path, and what that path is relative to the owning repo.
_Avoid_: mapping, dispatching, resolving

**Aggregate path**:
A path that expands to more than one child repo — the VMR root itself, as in `add -A`. "Aggregate" is reserved for this routing sense alone.
_Avoid_: root path, wildcard path, aggregate worktree (say worktree root)

**Scope**:
What a path-taking command operates over: explicit paths, or the entire VMR via the aggregate path.
_Avoid_: target, selection

**Target**:
A user-supplied path resolved against the working dir and normalized — the only form in which user paths reach the filesystem or routing.
_Avoid_: raw path, input path

**Move plan**:
The validated description of a mv: every source routed and
tracking-verified, every final destination resolved and conflict-checked,
built in full before anything moves. Execution consumes it entry by entry,
producing one repo outcome per entry.
_Avoid_: plan (unqualified), rename plan

### Worktrees

**Worktree root**:
A sibling VMR root materialized by `worktree add`, holding one child worktree per child repo at `<root>/<child repo name>`. The layout works both ways: a directory named after a child repo whose parent is a VMR root is that root's child worktree.
_Avoid_: aggregate worktree, aggregate directory

**Child worktree**:
The worktree a worktree root holds for one child repo, living at `<root>/<child repo name>`.
_Avoid_: linked worktree, sub-worktree

**Materialize / dissolve**:
The worktree-root lifecycle: materializing creates the directory and its marker; dissolving removes the marker and the directory if empty.
_Avoid_: create/cleanup (for this lifecycle)

### Running git

**Git runner**:
The single seam through which every git invocation passes. Two adapters satisfy it: the subprocess runner in production and the scripted fake in tests.
_Avoid_: git client, executor, git wrapper

**Scripted fake**:
The test adapter for the Git runner — answers each expected invocation with a canned exit code and output, fails the test on an unexpected one, and records what was run.
_Avoid_: mock git, stub

**Repo outcome**:
The per-repo result of a git operation: success (with an optional message) or failure (with one). The unit that result aggregation consumes.
_Avoid_: result, status, exit

**Report policy**:
The declarative per-operation rule for how a repo outcome's message is produced: which output streams are read in which order, whether an empty result means quiet success or a canned fallback, and any transform applied to the message before it reports.
_Avoid_: message extraction, message rules, output handling

**Result aggregation**:
Grouping repo outcomes with identical messages, so a command reports once per message across many child repos, not once per repo.
_Avoid_: output grouping, deduplication

**Head**:
Where a child repo currently points: a branch, or a detached commit.
_Avoid_: current branch, HEAD state

### Output

**Rendering**:
Turning per-repo data and outcomes into styled terminal text. Rendering is pure; printing the rendered text happens once, at a single choke point.
_Avoid_: formatting, displaying, output (for the act of rendering)

**Repo-list suffix**:
The gray parenthesised list of child repo names — "(backend, frontend)" — appended to a line that does not apply to every child repo in scope, and omitted when it does: no suffix means everyone. Truncated to three names plus a count ("(a, b, c, +2)") except on failures, which name every repo.
_Avoid_: repo annotation, repo tag

### Analytics

**Event metadata**:
The record of what was invoked: the dotted command name and the names of flags explicitly given on the command line, derived generically from the CLI definition. Names only, never values; positionals excluded.
_Avoid_: telemetry payload, event props, flag list

### Persistence

**FileStore**:
The single owner of one persistent TOML file: it holds the file's contents in memory, knows the file's path, and is the only thing that reads or writes it.
_Avoid_: store (unqualified), persistence layer, repository

**Global config**:
User-level mutable configuration from the user's config directory. Strict: a malformed file is an error, a missing one means defaults.
_Avoid_: settings

**VMR config**:
Per-VMR configuration living inside the main root's `.gitvmr` marker directory; created with defaults by init and never overwritten by it.
_Avoid_: local config, repo config

**Global state**:
User-level mutable bookkeeping (analytics session, update checks) from the user's state directory. Forgiving: a malformed file is replaced with defaults, and a warning is surfaced to the user.
_Avoid_: cache, app data
