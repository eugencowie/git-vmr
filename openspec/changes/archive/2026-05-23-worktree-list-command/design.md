## Context

`git vmr worktree add` creates a linked aggregate worktree by creating one child Git worktree per immediate child repository under a shared aggregate root and writing a `.gitvmr` marker at that root. `remove` and `move` operate on the same aggregate layout by accepting the aggregate root once and delegating child operations to Git.

`git vmr worktree list` should complete that workflow by showing users the aggregate worktrees that Git knows about across child repositories. Unlike `add`, `remove`, and `move`, listing needs to read worktree metadata from each child repository and combine it into one VMR-level view.

## Goals / Non-Goals

**Goals:**
- Add a nested `worktree list` command that mirrors Git's no-argument list subcommand shape.
- Discover worktree state by asking each child Git repository for its own `git worktree list --porcelain -z` output.
- Render a deterministic aggregate list grouped by VMR worktree root.
- Show branch or detached HEAD state, and show repository coverage when the state applies to only some child repositories.
- Preserve existing VMR root discovery, `-C`, and immediate-child repository scanning behavior.

**Non-Goals:**
- Add user-facing `-v`, `--porcelain`, `-z`, or other `git worktree list` options.
- List arbitrary child worktrees that are not laid out as `<aggregate-root>/<repo-name>` or that do not correspond to a marked VMR aggregate root.
- Infer or repair broken worktree metadata.
- Change `worktree add`, `remove`, or `move` behavior.

## Decisions

### Decision: Parse Git's porcelain list format internally

Use `git worktree list --porcelain -z` in each child repository, then parse records into a small internal worktree model containing path, HEAD, branch or detached state, and metadata flags such as bare, locked, and prunable when present.

Rationale: the porcelain format is designed for scripts and is more stable than column-aligned human output. Using `-z` avoids path parsing edge cases involving newlines.

Alternative considered: parse default `git worktree list` output. That output is intended for humans, has alignment concerns, and would force fragile parsing of paths and bracketed branch details.

### Decision: Render aggregate worktree groups by parent path

For a child repository named `backend`, treat a Git worktree at `<path>/backend` as belonging to aggregate root `<path>`. The main VMR root is included when the child worktree path equals the child repository path under the discovered VMR root. Ignore child worktrees that do not match either aggregate shape.

Rationale: `git-vmr` owns the aggregate layout created by `worktree add`, `remove`, and `move`. Listing the same layout gives users a coherent VMR-level view without exposing unrelated per-repository worktrees.

Alternative considered: print every child repository worktree with repository suffixes. That would be closer to raw Git, but it would not answer the VMR question users have: which aggregate worktrees exist and which repositories participate in them?

### Decision: Require aggregate roots to be VMR roots

Only render aggregate roots that are discoverable as VMR roots through a `.gitvmr` marker, plus the current main VMR root. This filters out unrelated sibling directories that happen to have matching child worktree basenames.

Rationale: aggregate linked worktrees created by `git-vmr` carry `.gitvmr`, and VMR root discovery depends on that marker. Restricting the list to marked roots avoids surprising output from unrelated child Git worktrees.

Alternative considered: include any parent path with at least one matching child worktree. That would reveal more raw Git state, but it could misclassify independent child worktrees as aggregate VMR worktrees.

### Decision: Group by path and head state

Render one line for each aggregate root and shared head state. If every child repository in a line has the same state, omit the repository list. If only some repositories share that state, append their names. When one aggregate root has mixed branches or detached states, render multiple lines for the same path with repository lists.

Rationale: existing `branch` and `status` output already compresses shared state and uses repository suffixes to show partial coverage. Reusing that model keeps mixed aggregate state visible without inventing a complex table.

Alternative considered: render one row per aggregate root with a dense branch summary column. That would be compact for fully aligned worktrees but harder to read and test for mixed states.

### Decision: Keep list read-only and fail on unreadable child metadata

The command should scan all immediate child Git repositories, attempt to read worktree lists in parallel, and fail if any child repository's worktree metadata cannot be read. It should not mutate aggregate markers or Git worktree metadata.

Rationale: read failures indicate the list may be incomplete. Failing clearly is preferable to presenting a partial inventory as authoritative.

Alternative considered: best-effort listing with repository-suffixed warnings. Existing mutating commands are best-effort because partial success is useful; for a read-only inventory, a clean failure is less ambiguous.

## Risks / Trade-offs

- [Risk] Git porcelain fields can grow over time. -> Mitigation: parse the fields needed for rendering and ignore unknown fields.
- [Risk] Filtering by `.gitvmr` can hide manually assembled aggregate worktrees without markers. -> Mitigation: this matches existing VMR discovery requirements and the aggregate worktree contract from `worktree add`.
- [Risk] Mixed branch states can produce repeated aggregate paths. -> Mitigation: repository suffixes make the split explicit and deterministic.
- [Risk] Paths outside UTF-8 can be difficult to render. -> Mitigation: store paths as `PathBuf` from raw porcelain bytes where practical, and render with `display()` like existing path output.
