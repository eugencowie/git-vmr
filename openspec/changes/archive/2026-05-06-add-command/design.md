## Context

`git-vmr` is an early-stage Rust CLI with `init` and `status` commands. The CLI resolves an effective working directory once, using either cwd or the global `-C <path>` flag, then passes that path into subcommands. `status` already discovers the VMR root by walking ancestors for `.gitvmr/`, scans immediate child directories, skips non-Git directories, and renders file paths relative to the effective working directory.

`git vmr add` needs to invert that status path model. A user-facing path is written relative to the effective working directory, but the actual Git operation must run inside the child repository that owns the path.

## Goals / Non-Goals

**Goals:**
- Add `git vmr add <path>...` for staging changes across immediate child Git repositories.
- Interpret path arguments relative to the same effective working directory used by existing commands.
- Route each path to exactly one child repository, or to all child repositories when the path is the VMR root subtree.
- Stage modified, untracked, and deleted paths.
- Preflight path ownership before running any `git add` command, reducing accidental partial staging.
- Keep behavior close to Git's path semantics where practical.

**Non-Goals:**
- Full Git pathspec magic such as `:(exclude)`, `--pathspec-from-file`, or shell glob expansion beyond what the user's shell already performs.
- Recursive discovery of nested repositories beyond immediate children of the VMR root.
- Transactional staging across repositories.
- New configuration for repo inclusion/exclusion.
- Implementing `restore`, `commit`, or other commands.

## Decisions

### Decision: Resolve add paths lexically, not with filesystem canonicalization

Path arguments will be joined to the effective working directory when relative, or used as-is when absolute, then normalized lexically by processing `.`, `..`, and normal components. The command will not require each target path to exist on disk before routing it.

This is necessary because deleted files must be stageable. `std::fs::canonicalize()` would reject deleted paths before Git can stage their removal. Lexical normalization also allows directory paths and repo-root paths to be routed without forcing existence checks outside Git.

Alternative considered: canonicalize every path argument. This is simpler for containment checks, but it breaks deleted-file staging and would diverge from normal `git add <deleted-path>` behavior.

### Decision: Route by immediate child repository ownership

After lexical normalization, paths must remain inside the VMR root and outside `.gitvmr/`. The first path component under the VMR root identifies the candidate child repository. If that child has a `.git` entry, the path is routed to that repository with the remainder as the repo-relative path. If the path names the child repository itself, the repo-relative path is `.`.

The special case is a path that resolves to the VMR root itself, such as `.` when invoked from the VMR root. That path expands to `.` for each immediate child Git repository and skips non-Git directories, matching the way `status` treats the root as an aggregate view.

Alternative considered: pass paths directly to a single Git command from the current repository. Git rejects paths outside the current repository, so this cannot stage sibling repositories.

### Decision: Preflight all paths before invoking Git

The command will resolve and route every path argument first. If any path is outside the VMR, inside `.gitvmr/`, or not owned by an immediate child Git repository, the command fails before staging anything.

This does not make the command transactional, but it avoids predictable partial updates caused by invalid path arguments.

Alternative considered: stream each path directly into `git add` as soon as it is parsed. That is simpler but creates avoidable partial staging when later paths fail validation.

### Decision: Group routed paths by repository and shell out to Git

The implementation should group repo-relative paths by repository, then invoke `git --no-optional-locks -C <repo> add -- <paths...>` once per repository. This matches the existing implementation style in `status`, which shells out to `git` for porcelain behavior, and avoids adding a Git library dependency.

Repos can be processed sequentially for predictable behavior. Parallel execution is not necessary for the initial `add` command because staging mutates indexes and the dominant UX concern is clear error handling.

Alternative considered: use a Rust Git library. The project does not currently depend on one, and staging semantics are easy to delegate to the Git CLI.

### Decision: Keep pathspec support literal-first

The first version should treat arguments as paths, not as full Git pathspec syntax. This keeps VMR ownership routing deterministic. Shell-expanded globs are still naturally supported because the command receives expanded paths from the shell.

Alternative considered: preserve arbitrary Git pathspecs and evaluate them inside each repository. That may be useful later, but cross-repo routing for pathspec magic is ambiguous and can produce surprising matches.

## Risks / Trade-offs

- **Cross-repo staging is not atomic** -> Preflight all path ownership before invoking Git; document that a later Git failure can still leave earlier repositories staged.
- **Lexical normalization is security-sensitive** -> Keep containment checks explicit: normalized targets must stay under the VMR root and must not enter `.gitvmr/`.
- **Literal path handling excludes advanced Git pathspecs** -> Start with predictable file and directory staging; add explicit pathspec support later if users need it.
- **Root `.` can stage many repositories** -> This is consistent with the aggregate VMR model, but tests should make the behavior explicit.
- **Non-Git child directories are skipped for root expansion but rejected when named directly** -> This mirrors status aggregation while still surfacing likely user mistakes for explicit paths.
