# git-vmr

A Git CLI wrapper for treating multiple independent repositories as a single unified workspace — the flexibility of independent repositories with the ergonomics of a monorepo.

## Language

### Workspace

**VMR**:
A virtual monorepo — a directory that groups independent Git repositories into one workspace, identified by a `.gitvmr` marker.
_Avoid_: workspace, monorepo, root repo

**Child repo**:
A Git repository living directly inside a VMR, operated on as part of the whole.
_Avoid_: submodule, subrepo, member, project

**Working dir**:
The directory a command is invoked from, or the override given with `-C`. Determines how user-supplied paths are interpreted.
_Avoid_: cwd, current directory

### Paths

**Routing**:
Deciding which child repo owns a user-supplied path, and what that path is relative to the owning repo.
_Avoid_: mapping, dispatching, resolving

**Aggregate path**:
A path that expands to more than one child repo — the VMR root itself, as in `add -A`.
_Avoid_: root path, wildcard path

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
Grouping repo outcomes with identical messages so a command reports once per message across many child repos, not once per repo.
_Avoid_: output grouping, deduplication

**Head**:
Where a child repo currently points: a branch, or a detached commit.
_Avoid_: current branch, HEAD state
