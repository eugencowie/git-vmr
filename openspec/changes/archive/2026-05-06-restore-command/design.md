## Context

`git-vmr` currently supports `init`, `status`, `add`, `mv`, and `rm`. Mutable commands resolve an effective working directory from cwd or the global `-C <path>` flag, discover the VMR root, lexically route VMR-facing paths to immediate child Git repositories, and then delegate repository-local behavior to Git where possible.

`git vmr status` already renders hints for `git vmr restore --staged <file>...` and `git vmr restore <file>...`, so the command should make those advertised workflows real. The command should be scoped to restore targets, not arbitrary tree-ish restores; `--source` is intentionally out of scope.

## Goals / Non-Goals

**Goals:**
- Add `git vmr restore <pathspec>...` for discarding unstaged tracked working tree changes.
- Add `--staged`, `--worktree`, and combined `--staged --worktree` restore targets.
- Preserve Git's restore semantics inside each child repository by invoking `git restore`.
- Reuse the established VMR path routing model, including relative paths from cwd or `-C`.
- Expand a VMR-root operand to all immediate child Git repositories for aggregate restore workflows.
- Validate all VMR path ownership before invoking any mutating Git restore command.

**Non-Goals:**
- Supporting `--source`, revision arguments, or restoring from arbitrary commits.
- Supporting advanced Git pathspec options such as `--pathspec-from-file`.
- Removing untracked files. `git restore` does not clean untracked files; users need Git clean behavior for that.
- Synthesizing cross-repository behavior beyond routing independent pathspecs to their owning repositories.

## Decisions

### Decision: Model restore as routed `git restore`

The command should group routed pathspecs by child repository and invoke Git once per repository:

```sh
git --no-optional-locks -C <repo> restore [--staged] [--worktree] -- <paths...>
```

When neither `--staged` nor `--worktree` is provided, the command should omit both flags and let Git perform its normal working-tree restore behavior. This keeps default behavior aligned with `git restore <pathspec>...`.

Alternative considered: implement restore with lower-level `git checkout-index`, `git reset`, and filesystem operations. That would duplicate Git porcelain semantics and risk diverging on staged additions, deletions, conflicts, and pathspec errors.

### Decision: Do not support `--source` in the initial command

The initial command should focus on the two workflows already advertised by `status`: unstage changes and discard working tree edits. `--source` introduces tree-ish parsing, source defaults, and more opportunities for ambiguity when users are operating from an aggregate VMR view.

Alternative considered: pass `--source` through directly to each child repository. That would be convenient for advanced Git users, but it expands the command's semantic contract before the core restore workflow exists.

### Decision: Reuse add-style lexical routing and validation

Path arguments should resolve relative to the effective working directory and normalize lexically without requiring every path to exist. This is important because restore often targets deleted paths, staged paths, or paths whose working-tree state no longer exists.

All path arguments should be validated before invoking Git. Invalid ownership includes paths outside the VMR root, paths under `.gitvmr/`, paths directly under the VMR root, and paths owned by non-Git child directories.

Alternative considered: canonicalize path arguments before routing. Canonicalization fails for deleted paths and would make restore less useful than Git's own pathspec handling.

### Decision: Expand VMR-root operands across child repositories

When a path resolves to the VMR root itself, such as `git vmr restore .` from the root, the command should route `.` to every immediate child Git repository and skip non-Git child directories. This mirrors `git vmr add .` and fits the aggregate status model.

Unlike `git vmr rm -r .`, no additional recursive flag should be required. Restore can discard work, but it does not recursively remove clean tracked files by itself; the target scope is already visible in the `restore .` pathspec.

Alternative considered: reject the VMR root aggregate for restore, as `mv` does. That would make broad discard and unstage workflows awkward and inconsistent with status presenting a whole-VMR view.

### Decision: Keep unsupported restore flags rejected by clap

The CLI should only expose `--staged` and `--worktree` for this change. Unsupported flags such as `--source`, `--patch`, `--ours`, `--theirs`, and pathspec file options should fail during argument parsing instead of being silently forwarded.

Alternative considered: accept arbitrary trailing flags and pass them to Git. That would reduce implementation work, but it would also make VMR-level behavior under-specified and harder to test.

## Risks / Trade-offs

- `git vmr restore .` from the VMR root can discard unstaged work across many repositories -> Make root expansion explicit in the spec and cover it with tests.
- Partial mutation can occur if one repository restore succeeds and a later repository fails -> Preflight VMR ownership for all paths first, process repositories deterministically, and report the failing child repository.
- Git restore behavior differs by target flags and path state -> Delegate to Git rather than reimplementing semantics, and test representative staged, unstaged, deleted, and multi-repository cases.
- Skipping `--source` may disappoint advanced users -> Keep the initial command focused; add `--source` later with a separate proposal if real usage needs it.
