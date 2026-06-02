## Context

git-vmr is an early-stage virtual monorepo manager with a single `init` command that creates a `.gitvmr/` marker directory. The CLI uses clap (derive) with a global `-C` flag for working directory override. There are no git operations yet — no git library dependency exists. The codebase is structured as `src/cli/<command>.rs` modules with a shared `src/config/` for configuration types.

## Goals / Non-Goals

**Goals:**
- Provide a `git vmr status` command that shows unified git status across all repos in a VMR
- Group output by branch name so divergence is immediately visible
- Output paths relative to the current working directory
- Execute status collection in parallel across repos
- Introduce a reusable VMR root discovery function for future commands

**Non-Goals:**
- Tracking state (ahead/behind, up-to-date) — omitted entirely
- Recursive subdirectory traversal — only immediate children
- Config-driven repo filtering or exclusions
- Modifying `.gitvmr/config` schema for status settings

## Decisions

### Decision: Use `gix` for git operations

Shell out to `git` binary vs. use `gix` crate. Chose `gix` because:
- No dependency on git being installed at runtime
- Structured access to repo state (branch, status, untracked files)
- Composable with rayon without subprocess overhead
- Full control over output formatting

### Decision: Use `rayon` for parallel status collection

Sequential vs. parallel. Chose `rayon::par_iter` because:
- Status collection across repos is embarrassingly parallel
- `rayon` is the standard Rust parallelism library — minimal API surface
- Thread pool is managed automatically
- Results are collected into a `Vec` and rendered sequentially for deterministic output

### Decision: Two-phase execution (collect then render)

Run all repo introspection in parallel, collect structured results, then render in sorted order by repo name. This ensures:
- Deterministic output regardless of thread scheduling
- Single-pass rendering with correct branch grouping
- No interleaved output from parallel threads

### Decision: VMR root discovery as a standalone function

`find_vmr_root(start: &Path) -> Result<PathBuf>` walks ancestors from `start` looking for `.gitvmr/`. Returns the parent of the `.gitvmr/` directory. This will be reused by every future VMR command (clone, pull, etc.).

### Decision: Branch grouping key

Repos are grouped by their branch name for rendering. Detached HEAD repos each get their own section with `"HEAD detached at <short-hash> (<repo-name>)"`. Repos with no commits yet show `"On branch <name> (<repo-name>)"` followed by `"No commits yet."` When only one unique branch exists across all repos, the branch header omits the repo list — output is indistinguishable from regular `git status`.

### Decision: cwd-relative path computation

File paths from `gix` are relative to the repo root. These are re-based to be relative to the working directory (the resolved `-C` or cwd value). This means running from inside a repo shows that repo's paths without prefix, while sibling repo paths get `../repo-name/` prefixes.

### Decision: Colored file entries using `anstyle` and `anstream`

Use `anstyle` for styling definitions and `anstream` for auto-detecting tty support (color when interactive, strip when piped). Chose these over `colored` or `owo-colors` because:
- Already transitive dependencies of clap — no new dependency weight
- `anstream` handles tty detection and `NO_COLOR`/`TERM` environment conventions automatically
- `anstyle` provides a composable, zero-cost style model

Only file entry lines are colored (green for staged, red for unstaged and untracked). Branch headers, section headers, and status labels remain uncolored.

## Risks / Trade-offs

- **`gix` crate size and compile time** → `gix` is a large crate. Mitigate by enabling only needed features (status, worktree, no network/remote features). Accept the cost as git operations are central to the tool.
- **`gix` status API maturity** → `gix`'s status API may not match git's porcelain output exactly. May need iterative refinement to match expected output. Start with what `gix` provides and adjust.
- **No tracking state** → Users accustomed to `git status` showing ahead/behind will not see this. Can be added later without breaking changes.
- **Immediate children only** → Nested monorepo structures are not supported. This is a deliberate scope limit — can be relaxed later with a config option or flag.
