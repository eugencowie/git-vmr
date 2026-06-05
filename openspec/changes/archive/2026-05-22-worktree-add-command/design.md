## Context

`git-vmr` is installed as `git-vmr`, so users can run `git vmr <subcommand>`. Existing VMR fan-out commands discover the VMR root from the effective working directory, scan immediate child Git repositories, skip non-Git child directories, and invoke Git in each child repository. Existing path-routing commands resolve relative operands against the effective working directory and validate VMR ownership before mutation.

`git vmr worktree add <path> [<commit-ish>]` combines both patterns. It is a VMR-level creation command whose target path is an aggregate directory outside or inside the current working directory, while each actual Git worktree is created inside that aggregate target at `<path>/<repo-name>`.

## Goals / Non-Goals

**Goals:**
- Add a nested `worktree add` command shape that mirrors Git's `worktree add` entry point.
- Create a linked aggregate VMR worktree with one child Git worktree per immediate child repository.
- Preserve Git's own worktree safety checks, commit-ish resolution, output text, and failure text.
- Infer a single aggregate branch name from `<path>` only when `<commit-ish>` is omitted.
- Use best-effort execution across child repositories without rolling back successful worktree additions.
- Make the new aggregate path discoverable as a VMR by creating a `.gitvmr` marker.

**Non-Goals:**
- Support other `git worktree` subcommands such as `list`, `remove`, `prune`, `lock`, or `move`.
- Forward arbitrary `git worktree add` flags such as `--detach`, `--orphan`, `--guess-remote`, or `--track`.
- Implement transactional rollback for partial failures.
- Add persistent VMR metadata beyond the root marker required for discovery.

## Decisions

### Decision: Model `worktree` as a nested command

Add a top-level `Worktree` command with an `Add { path, commit_ish }` action rather than a flat `WorktreeAdd` command.

Rationale: users invoke Git as `git worktree add`, and this keeps room for future VMR worktree subcommands without changing the top-level command surface later.

Alternative considered: add a flat `worktree-add` style command. That would be simpler internally, but it would not match Git's command shape and would make future worktree support awkward.

### Decision: Resolve aggregate target once, then append child repository names

The command should resolve `<path>` against the effective working directory. For each child repository, it should use `<resolved-target>/<repo-name>` as the Git worktree path.

Rationale: this creates a linked VMR with the same immediate child layout as the source VMR, and it matches existing `-C` behavior where command-specific paths are interpreted relative to the resolved working directory.

Alternative considered: interpret `<path>` relative to the VMR root. That would make invocations from nested child directories less predictable and would be inconsistent with `clone` and `init <directory>` path handling.

### Decision: Infer one aggregate branch only when `<commit-ish>` is omitted

For `git vmr worktree add ../wt`, derive `wt` from the aggregate target basename and call Git in each child repository with an explicit new branch, equivalent to `git worktree add -b wt <target>/<repo-name>`.

Rationale: plain Git would infer the branch name from each child path, producing branches named `backend`, `frontend`, and so on. The VMR command needs the aggregate path basename to become the shared branch name across child repositories.

Alternative considered: delegate omitted `<commit-ish>` directly to Git. That would preserve Git's local behavior per child path, but it would create inconsistent branch names and violate the aggregate VMR mental model.

### Decision: Pass explicit `<commit-ish>` directly to Git

For `git vmr worktree add ../wt main`, call Git in each child repository with `git worktree add <target>/<repo-name> main`. Do not create a branch named from the aggregate target in this mode.

Rationale: this preserves Git's distinction between creating a new branch when no commit-ish is provided and checking out the requested commit-ish when it is provided. It also lets Git produce standard errors such as a branch already being checked out in another worktree or an invalid reference.

Alternative considered: always create a new aggregate branch from the explicit commit-ish, equivalent to `git worktree add -b wt <path> main`. That is useful behavior, but it does not match the requested command semantics and can be added later with an explicit flag if needed.

### Decision: Best-effort execution with no rollback

The command should attempt every child repository and report failures after all attempts complete. Successful worktree additions remain in place if other repositories fail.

Rationale: this matches existing VMR fan-out command behavior and avoids implementing fragile cleanup logic around Git worktree state. Users can inspect and repair partial aggregate worktrees with normal Git commands.

Alternative considered: all-or-nothing rollback. That could produce a cleaner aggregate target after failure, but rollback can itself fail and risks hiding the original Git error behind cleanup behavior.

### Decision: Create a lightweight `.gitvmr` marker in the aggregate target

The command should ensure the aggregate target has a `.gitvmr` marker so existing VMR root discovery works from the linked worktree. The marker can be a file because root discovery already accepts `.gitvmr` as either a directory or a file.

Rationale: child worktrees need `git vmr status`, `git vmr branch`, and future commands to discover the linked aggregate root. A marker file avoids copying or inventing config content for the linked worktree.

Alternative considered: copy the source `.gitvmr/config` directory. That would preserve config bytes, but it is unnecessary for current discovery and risks creating multiple independent configuration files before config semantics are defined for linked VMR worktrees.

## Risks / Trade-offs

- [Risk] Partial failures leave an incomplete aggregate worktree. -> Mitigation: report repository-suffixed failures deterministically and document that the command is best-effort.
- [Risk] A pre-created `.gitvmr` marker may remain if all child worktree additions fail. -> Mitigation: keep marker creation simple and let users remove the target path manually, consistent with no rollback.
- [Risk] Branch-name inference from unusual target paths can produce invalid branch names. -> Mitigation: pass the inferred name to Git and let Git validate branch naming consistently per child repository.
- [Risk] Parallel Git worktree additions may interleave progress output if inherited directly. -> Mitigation: capture per-repository output and render the first non-empty line through the existing aggregate reporting pattern.
