## Context

`git-vmr` currently supports `init`, `status`, `add`, and `rm`. The CLI resolves an effective working directory once, using either cwd or the global `-C <path>` flag, then mutating subcommands discover the VMR root and route user-facing paths to immediate child Git repositories.

`git vmr add` and `git vmr rm` established the mutable command model: resolve paths relative to the effective working directory, normalize lexically, validate VMR ownership before mutation, and delegate Git semantics to child repositories. `git vmr mv` should use the same routing model, but moving between repositories cannot be delegated to a single `git mv` because each child repository has its own worktree and index.

## Goals / Non-Goals

**Goals:**
- Add `git vmr mv <source> <destination>` for moving one path within or between immediate child Git repositories.
- Interpret both operands relative to the same effective working directory used by existing commands.
- Preserve Git's same-repository move behavior by invoking `git mv` inside the owning child repository.
- Support cross-repository moves by moving the worktree path and staging the source deletion plus destination addition.
- Validate both operands and predictable Git preconditions before moving anything.
- Support destination-directory behavior compatible with `git mv <source> <destination-directory>`.

**Non-Goals:**
- Multi-source moves such as `git mv a b dir/`.
- `git mv` flags such as `--dry-run`, `-k`, `--sparse`, or `-f` in the initial command.
- Moving paths into or out of `.gitvmr/`, root-level VMR files, or non-Git child directories.
- Recursive discovery of nested repositories beyond immediate children of the VMR root.
- Transactional moves across child repositories.

## Decisions

### Decision: Keep the initial CLI to two operands

The first version should expose `git vmr mv <source> <destination>`. Clap can enforce exactly one source and one destination, giving a clear command shape that matches the most important `git mv` workflow while avoiding ambiguous VMR routing for multi-source destination-directory moves.

Alternative considered: support `git mv <source>... <destination-directory>` parity immediately. That is useful, but it complicates preflight validation and cross-repo rollback without changing the core routing model. It can be added later as an extension.

### Decision: Reuse lexical VMR routing for both operands, without root expansion

Both operands will be resolved relative to the effective working directory, normalized lexically, and checked for VMR containment, `.gitvmr/` exclusion, and immediate child Git repository ownership. Unlike `add` and recursive `rm`, `mv` should not expand a path that resolves to the VMR root into all child repositories. The VMR root is an aggregate view, not a concrete move source or destination.

This suggests either adding a routing helper for exactly-one-owned path or extending the existing router with an option that rejects aggregate root expansion.

Alternative considered: reuse `route_path` directly and inherit VMR-root expansion. That would make `git vmr mv . frontend/all` ambiguous and risky, because one destination cannot receive an aggregate set of repositories in a Git-like way.

### Decision: Delegate same-repository moves to `git mv`

When source and destination route to the same child repository, the implementation should run:

```sh
git --no-optional-locks -C <repo> mv -- <source-relative> <destination-relative>
```

This keeps tracked-file checks, destination-exists checks, directory handling, case sensitivity behavior, and index/worktree updates aligned with Git.

Alternative considered: implement same-repository moves with filesystem rename plus `git add`. That would duplicate Git porcelain behavior and risk diverging on edge cases that Git already handles.

### Decision: Synthesize cross-repository moves with preflight, filesystem move, and staging

When source and destination route to different child repositories, there is no single `git mv` operation that can update both indexes. The command should synthesize the equivalent VMR effect:

1. Preflight source tracking with Git in the source repository.
2. Resolve destination-directory behavior before the move.
3. Move the filesystem path from the source repo to the destination repo.
4. Stage the source path in the source repository with `git add -- <source-relative>` so the deletion is recorded.
5. Stage the final destination path in the destination repository with `git add -- <destination-relative>` so the addition is recorded.

The resulting committed history is a deletion in one repository and an addition in another. That is the closest possible equivalent to `git mv` across independent Git repositories.

Alternative considered: invoke `git rm` in the source repo and then copy content into the destination repo. That would fail to preserve directories and metadata as naturally as a filesystem move and would remove the source before destination creation.

### Decision: Match destination-directory semantics before cross-repo moves

For cross-repository moves, if the destination operand resolves to an existing directory, including a child repository root, the final destination should be `<destination>/<source-basename>`, matching `git mv <source> <destination-directory>`. Otherwise, the destination operand is treated as the exact final path and its parent directory must already exist.

This keeps `git vmr mv backend/a.rs frontend` useful and predictable: the final path is `frontend/a.rs`.

Alternative considered: require destination operands to always name the final path. That is simpler, but it drops a core `git mv` behavior and makes repo-root destinations awkward.

### Decision: Validate predictable failure cases before mutation

Before any move, the command should validate both operands route to child Git repositories and that the source is tracked by Git. For cross-repository moves, it should also validate destination shape enough to avoid avoidable partial moves: reject an existing non-directory destination, reject a missing destination parent, and reject root aggregate operands.

The command is still not fully transactional. Filesystem and Git operations can fail after preflight because of permissions, index locks, filesystem races, or Git version behavior. Error messages should include the relevant child repository path and operation context.

Alternative considered: rely on the post-move `git add` calls to surface every issue. That would leave obvious cases, such as untracked sources, to fail only after the worktree has already changed.

## Risks / Trade-offs

- **Cross-repository moves are not atomic** -> Preflight ownership, source tracking, and destination shape before moving; keep operations sequential and report the failing repository or path.
- **Cross-repo history cannot record a Git rename** -> Document and test that the observable state is a staged deletion in the source repo and staged addition in the destination repo.
- **Destination semantics can diverge from Git on edge cases** -> Delegate same-repo moves to `git mv`; for cross-repo moves, cover destination directory, existing destination, and missing parent behavior with tests.
- **Routing code may grow awkward if `route_path` always expands the VMR root** -> Prefer a small shared helper for single-owner routing rather than duplicating lexical containment logic in `mv`.
- **Case-only renames and platform differences** -> Same-repo cases stay delegated to Git; cross-repo tests should avoid depending on case-insensitive filesystem behavior.
