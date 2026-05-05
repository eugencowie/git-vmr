## Context

`git-vmr` currently has `init`, `status`, and `add` commands. The CLI resolves an effective working directory once, using either cwd or the global `-C <path>` flag, then subcommands discover the VMR root by walking ancestors for `.gitvmr/`.

`git vmr add` established the path-routing model for mutating child repository indexes: resolve user-facing paths relative to the effective working directory, normalize them lexically, validate VMR ownership up front, group repo-relative paths by immediate child Git repository, and shell out to Git once per repository. `git vmr rm` should use the same ownership model while delegating removal semantics to Git's `rm` command.

## Goals / Non-Goals

**Goals:**
- Add `git vmr rm <path>...` for removing tracked paths from child repositories.
- Interpret path arguments relative to the same effective working directory used by existing commands.
- Route each path to exactly one child repository, or to all child repositories when the path is the VMR root and recursive removal is explicitly requested.
- Preflight path ownership before running any `git rm` command, reducing avoidable partial removals.
- Support recursive removal through `-r` / `--recursive`.
- Keep behavior close to Git's own `rm` semantics where practical.

**Non-Goals:**
- Removing untracked files that Git would not remove.
- Supporting `git rm --cached`, `--force`, `--ignore-unmatch`, or pathspec-from-file options in the initial command.
- Full Git pathspec magic such as `:(exclude)` beyond shell-expanded literal paths.
- Recursive discovery of nested repositories beyond immediate children of the VMR root.
- Transactional removal across repositories.

## Decisions

### Decision: Model `rm` as routed `git rm`, not filesystem deletion

`git vmr rm` will remove tracked paths by invoking `git rm` inside the owning child repository. It will not behave like `/bin/rm`, and it will not delete arbitrary untracked files.

This matches the Git mental model already implied by `git vmr add`: the VMR command is a cross-repository router for Git index/worktree operations, not a replacement for shell filesystem commands.

Alternative considered: implement filesystem deletion directly and then stage with `git add`. That would blur Git semantics, create more edge cases around untracked files, and duplicate behavior Git already handles.

### Decision: Reuse add-style lexical routing and preflight validation

Path arguments will be joined to the effective working directory when relative, or used as-is when absolute, then normalized lexically by processing `.`, `..`, and normal components. The command will validate that every normalized target stays inside the VMR root, avoids `.gitvmr/`, and maps to an immediate child Git repository before invoking Git.

This mirrors `git vmr add` and keeps `rm` usable from any working directory inside the VMR, including sibling-repository paths such as `../backend/file.rs`.

Alternative considered: canonicalize every path argument. That can work for existing files, but it is inconsistent with the established add routing and can produce worse errors for paths that Git itself should diagnose.

### Decision: Require recursive intent for directory and VMR-root removal

The command will support only one rm-specific flag at first: `-r` / `--recursive`. When provided, it will pass recursive intent to `git rm`. When omitted, directory paths and the VMR-root aggregate path will be routed normally but Git should reject recursive removal.

The VMR root is special. A path that resolves to the VMR root, such as `git vmr rm .` from the root, represents removal across all immediate child Git repositories. That aggregate operation must require `-r` / `--recursive`; without it the command should fail before invoking Git.

Alternative considered: reject VMR-root `.` entirely for `rm`. That is safer but less symmetric with `add`, and it prevents an explicit recursive all-repos removal workflow. Requiring `-r` makes the destructive scope visible.

### Decision: Group routed paths by repository and shell out to Git

The implementation should group repo-relative paths by repository, then invoke `git --no-optional-locks -C <repo> rm [--recursive] -- <paths...>` once per repository. Repositories can be processed sequentially for predictable errors.

This matches the existing command style, avoids adding a Git library dependency, and lets Git enforce tracked-file, local-modification, and directory-removal rules.

Alternative considered: use a Rust Git library. The project already delegates porcelain behavior to the Git CLI for mutable operations, and `git rm` has well-understood failure modes that should be preserved.

### Decision: Keep pathspec support literal-first

The first version should treat arguments as paths, not as full Git pathspec syntax. Shell-expanded globs still work because the command receives expanded paths from the shell, but advanced Git pathspec forms are out of scope.

Alternative considered: preserve arbitrary Git pathspecs and evaluate them inside each repository. Cross-repo ownership routing for pathspec magic is ambiguous and can produce surprising matches.

## Risks / Trade-offs

- **Cross-repo removal is not atomic** -> Preflight all path ownership before invoking Git; document that a later Git failure can still leave earlier repositories changed.
- **`git vmr rm -r .` can remove many tracked files** -> Require explicit recursive intent for VMR-root expansion and make the behavior explicit in tests.
- **Git failure messages may vary by Git version** -> Tests should assert stable command-level behavior and key context rather than exact full stderr text.
- **Routing code duplication with `add`** -> Prefer extracting shared path routing if implementation would otherwise duplicate complex lexical containment logic.
- **Literal path handling excludes advanced Git pathspecs** -> Start with predictable file and directory removal; add explicit pathspec support later if users need it.
