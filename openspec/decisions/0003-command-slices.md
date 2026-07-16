---
status: accepted
---

# Commands are vertical slices over shared cores

Each command is one module — a command slice — owning the command end to end: its clap args, its orchestration, its git arg construction, and its report policy. The git operation lives inside its single caller as a private `impl Git` block: the method syntax is unchanged at the call site, but the compiler restricts it to the defining module. A slice depends only on the shared cores — git (runner seam, report engine, head resolution, output types), workspace (discovery, routing, run helpers), and render — never on another slice.

The rule that produced this shape: an operation lives with its single caller until a second caller earns its promotion to a core. That is why `git/mv.rs` and `git/worktree.rs` fused into `workspace/mv.rs` and `workspace/worktree_root.rs` (their callers were never the command files), and why `Git::add_path` — plumbing defined in the add operation but called only by move-plan execution — moved into `workspace/mv.rs` as a private method rather than staying public as speculative shared plumbing.

## Considered Options

- **Keep the commands/git two-module split per operation** — rejected: each `Git::<op>` method had exactly one caller and one adapter, so the seam was hypothetical, and the clap args had already migrated into the git operation files — the CLI surface was leaking across a seam that wasn't real.
- **`pub(crate)` operation methods called across the workspace/slice line** — rejected: it manufactures dependencies between slices that the compiler cannot police; private methods in the caller's module make the slices-never-depend-on-slices rule mechanical.
- **Free functions per slice instead of private `impl Git` blocks** — rejected: every call site changes shape for no gain in visibility; the private method achieves the same compiler-enforced locality with zero caller churn.

## Consequences

- The git core's interface shrinks to what is genuinely shared: the runner seam (ADR-0001, untouched), the report engine (policies stay in slices, ADR-0002), head resolution, invocation plumbing, and the output types.
- "What can `Git` do?" is no longer answerable from `git.rs` alone — the commands say what for. The core answers "run git and report outcomes".
- Per-command rendering lives in its slice as a private `render` function; the render core keeps what every slice shares — the `Rendered` type, the outcomes engine, the palette, and the repo-list suffix. The query result types moved with their render modules when those fused (`RepoStatus` and friends into status, `RepoBranches` into branch, `ChildOutput` into foreach).
- Future deepenings follow the same rule: a helper stays private to its caller until a second caller appears; promotion to a core is driven by callers, not speculation.
