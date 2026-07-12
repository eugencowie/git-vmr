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

### Worktrees

**Worktree root**:
A sibling VMR root materialized by `worktree add`, holding one linked child worktree per child repo.
_Avoid_: aggregate worktree, aggregate directory

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

**Result aggregation**:
Grouping repo outcomes with identical messages, so a command reports once per message across many child repos, not once per repo.
_Avoid_: output grouping, deduplication

**Head**:
Where a child repo currently points: a branch, or a detached commit.
_Avoid_: current branch, HEAD state

### Output

**Rendering**:
Turning per-repo data and outcomes into styled terminal text. Rendering is
pure; printing the rendered text happens once, at a single choke point.
_Avoid_: formatting, displaying, output (for the act of rendering)

**Repo-list suffix**:
The parenthesised list of child repo names — "(backend, frontend)" — appended
to a message or heading that does not apply to every child repo.
_Avoid_: repo annotation, repo tag
