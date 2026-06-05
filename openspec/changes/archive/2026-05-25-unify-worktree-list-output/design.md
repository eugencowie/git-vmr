## Context

`git vmr branch` and `git vmr tag` both collapse per-repository data into a VMR-level list: the shared logical item appears once, and repository suffixes appear only when the item applies to a subset of child repositories. `git vmr worktree list` already discovers aggregate worktree roots, but its renderer groups output by aggregate path, child state, and short HEAD hash. Because VMR child repositories usually have unrelated commit IDs, that grouping can repeat the same aggregate worktree path and expose per-repository Git internals in the default view.

The existing worktree list implementation should remain responsible for discovering child worktrees through `git worktree list --porcelain -z`, filtering to VMR aggregate layouts, and preserving deterministic output. This change only reshapes the human-facing renderer.

## Goals / Non-Goals

**Goals:**

- Present each aggregate worktree root as one unified VMR-level entry.
- Match the `branch` and `tag` command pattern: shared state is shown once, partial state gets repository suffixes.
- Group branch-backed child worktrees by branch name rather than by HEAD hash.
- Preserve useful detached HEAD reporting, including short commit identity where needed.
- Keep output deterministic for paths, state groups, and repository suffixes.

**Non-Goals:**

- Add new `git vmr worktree list` flags such as `--porcelain`, `-z`, or `-v`.
- Change aggregate worktree discovery, marker filtering, root discovery, or `-C` behavior.
- Change `worktree add`, `worktree remove`, or `worktree move`.
- Introduce a machine-readable output format.

## Decisions

### Decision: Render one aggregate entry per root

The renderer should collect all participating child worktree entries for an aggregate root, print the aggregate root once, and then render its state summary. When all participating child worktrees share a single branch, the compact entry can remain one line. When the aggregate root contains mixed branches or detached states, state-specific lines should be nested under the aggregate root rather than repeating the root path.

Rationale: the aggregate root is the VMR-level object users manage with `worktree remove` and `worktree move`. Showing it once makes the list scan like an inventory of aggregate worktrees.

Alternative considered: keep one row per state group and repeat the root path. That preserves a Git-like table shape, but it is the source of the inconsistency this change is addressing.

### Decision: Group branch-backed state by branch name

For child worktrees on a branch, branch name should be the grouping key. The renderer should not split rows or sub-lines for the same branch solely because child repositories point at different commit hashes.

Rationale: `git vmr branch` treats branch names as the cross-repository unit even though each child repository has its own commit graph. `worktree list` should use the same abstraction for branch-backed worktrees.

Alternative considered: include short HEAD in branch-backed output. That is closer to raw `git worktree list`, but it makes unrelated child repository hashes look like meaningful aggregate divergence in the default VMR view.

### Decision: Keep short HEAD only for detached state

Detached child worktrees have no branch name, so a short HEAD remains useful to distinguish detached states. Detached entries should be grouped by short HEAD plus repository coverage, and repository suffixes should clarify which child repositories are detached at each commit.

Rationale: without a branch name, the commit is the only concise state label. This preserves the useful part of current detached reporting while avoiding hash-based fragmentation for normal branch-backed worktrees.

Alternative considered: render all detached child repositories as a single `(detached HEAD)` state. That is simpler, but it hides meaningful differences between detached commits.

### Decision: Keep color and suffix behavior aligned with existing list commands

Repository suffixes should continue to be omitted when all participating child repositories for an aggregate root share the rendered state, and included in deterministic repository-name order when state applies only to a subset. If worktree list adopts colored suffixes, it should reuse the existing bright-black suffix convention from `branch` and `status`.

Rationale: this keeps the command family visually and semantically consistent without introducing a new table or column format.

Alternative considered: use a dense table with columns for root, state, and repositories. That would be explicit, but it would diverge from the established compact list pattern.

## Risks / Trade-offs

- [Risk] Changing human-facing output can break tests or user scripts that parse the current output. -> Mitigation: `worktree list` does not expose a supported porcelain mode through `git-vmr`; update tests and docs to define the new human format.
- [Risk] A nested mixed-state format is less compact for divergent aggregate worktrees. -> Mitigation: the common aligned-branch case stays compact, and mixed state becomes easier to scan because the root appears once.
- [Risk] Omitting branch HEAD hashes can hide per-repository commit differences. -> Mitigation: this matches the abstraction used by `branch` and `tag`; users can inspect per-repository commits with Git commands inside child repositories when needed.
