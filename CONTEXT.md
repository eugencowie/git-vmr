# git-vmr

A Git CLI wrapper for treating multiple independent repositories as a single unified workspace — the flexibility of independent repositories with the ergonomics of a monorepo.

## Language

### Workspace

**VMR**:
A virtual monorepo — a directory on disk that groups independent Git repositories, identified by a `.gitvmr` marker.
_Avoid_: monorepo, root repo

**Workspace**:
A VMR opened by one command: the working dir it was opened from and the child repos discovered at that moment, held fixed for the duration of the command, plus the means to run git across them. The disk can change mid-command; the workspace cannot.
_Avoid_: VMR (when the fixed, opened view is meant), context, session

**Child repo**:
A Git repository living directly inside a VMR, operated on as part of the whole.
_Avoid_: submodule, subrepo, member, project

**Workspace command**:
A command that runs inside an opened workspace — every command except clone
and init, which run before a VMR exists. The distinction is encoded in the
command type, not checked at run time.
_Avoid_: repo command, VMR command

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

**Router**:
The module that owns routing: a borrowed view over a workspace's fixed snapshot (VMR root, child repos, working dir) turning a scope into owning child repos with repo-relative paths. Private to the workspace core — slices reach routing only through the workspace.
_Avoid_: path resolver, route table

**Aggregate path**:
A path that expands to more than one child repo — the VMR root itself, as in `add -A`. "Aggregate" is reserved for this routing sense alone.
_Avoid_: root path, wildcard path, aggregate worktree (say worktree root)

**Aggregate policy**:
The declared per-command rule for what happens when a user-supplied path resolves to the aggregate path: allowed to expand across the workspace, or denied with a command-supplied message.
_Avoid_: aggregate guard, root check

**Scope**:
What a path-taking command operates over: explicit paths, or the entire VMR via the aggregate path.
_Avoid_: target, selection

**Target**:
A user-supplied path resolved against the working dir and normalized — the only form in which user paths reach the filesystem or routing.
_Avoid_: raw path, input path

**Move plan**:
The validated description of a mv: every source routed and tracking-verified, every final destination resolved and conflict-checked, built in full before anything moves. Execution consumes it entry by entry, producing one repo outcome per entry.
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

**Root outcomes**:
The result of a mutating root operation: one repo outcome per child worktree, plus the root's own fate — dissolved, deliberately kept, or failed to dissolve. Rendered as a single unit; a failed root fate reports as the command's error without discarding child successes.
_Avoid_: root result, operation epilogue

### Running git

**Git runner**:
The single seam through which every git invocation passes. Two adapters satisfy it: the subprocess runner in production and the scripted fake in tests.
_Avoid_: git client, executor, git wrapper

**Scripted fake**:
The test adapter for the Git runner — answers each expected invocation with a canned exit code and output, fails the test on an unexpected one, and records what was run.
_Avoid_: mock git, stub

**Child environment**:
The per-repo variable set foreach exports to the child's shell — name, sm_path, displaypath, and toplevel, with names taken from git submodule foreach's contract and sha1 deliberately absent. displaypath is relative to the working dir.
_Avoid_: env vars, shell context, foreach variables

**Repo outcome**:
The per-repo result of a git operation: success (with an optional message), failure (with one), or skipped (with a required reason — the operation was deliberately not attempted). The unit that result aggregation consumes.
_Avoid_: result, status, exit, filtered out or ignored (for skipped)

**Report policy**:
The declarative per-operation rule for how a repo outcome's message is produced: which output streams are read in which order, whether an empty result means quiet success or a canned fallback, and any transform applied to the message before it reports.
_Avoid_: message extraction, message rules, output handling

**Result aggregation**:
Grouping repo outcomes with identical messages, so a command reports once per message across many child repos, not once per repo. Consumes the outcomes together with the child repos in scope — supplied by the command, since outcomes cannot self-describe their scope — and omits the repo-list suffix when a group covers every repo in scope.
_Avoid_: output grouping, deduplication

**Head**:
Where a child repo currently points: a branch, an unborn branch (no commits yet), or a detached commit. Only the worktree record cannot observe unbornness; it reports a plain branch instead.
_Avoid_: current branch, HEAD state, initial (for unborn)

### Pushing

**Useful push**:
A push proven, from local remote-tracking refs alone, to do something: transmit novel commits, or update or delete a ref that already exists on the target remote. The push command attempts only useful pushes; anything unproven is skipped with a reason.
_Avoid_: necessary push, non-empty push

**Novel commits**:
Commits reachable from a local ref but not from any of the target remote's remote-tracking refs. The evidence that makes a push useful.
_Avoid_: new commits, unpushed commits

**Empty branch**:
A branch created on the remote whose tip carries no novel commits — the pollution artifact the useful-push filter exists to prevent.
_Avoid_: stale branch, no-op branch

### Output

**Rendering**:
Turning per-repo data and outcomes into styled terminal text. Rendering is pure; printing the rendered text happens once, at a single choke point.
_Avoid_: formatting, displaying, output (for the act of rendering)

**Repo-list suffix**:
The gray parenthesised list of child repo names — "(backend, frontend)" — appended to a line that does not apply to every child repo in scope, and omitted when it does: no suffix means everyone. Truncated to three names plus a count ("(a, b, c, +2)") except on failures, which name every repo.
_Avoid_: repo annotation, repo tag

### Invocation

**Run pipeline**:
The fixed sequence every invocation passes through after parsing: build context, report load warnings, execute and report the command, record analytics, check for updates, save state. Notices never fail the command; the exit code mirrors the command result.
_Avoid_: command lifecycle, post-command tail

### Analytics

**Analytics session**:
The rolling window that groups analytics events: the session ID is reused while activity stays within the window, and rotated once it expires.
_Avoid_: session (unqualified), tracking session

**Event metadata**:
The record of what was invoked: the dotted command name and the names of flags explicitly given on the command line, derived generically from the CLI definition. Names only, never values; positionals excluded.
_Avoid_: telemetry payload, event props, flag list

**Analytics sink**:
The seam through which every analytics emission passes, receiving a session ID and fully-built event props. Three adapters satisfy it: a file log, the hosted collector, and a no-op. Failures are swallowed at the seam, never in an adapter.
_Avoid_: transport, emitter, backend

### Updates

**Update check**:
The scheduled query for a newer version: due when the configured check frequency has elapsed, with the attempt recorded before the query, so failures don't repeat, and quiet on every failure path.
_Avoid_: version check, update poll

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
