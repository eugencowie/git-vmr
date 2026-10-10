# Commands are vertical slices over shared cores

Commands used to be split in two: a command module holding the CLI side, and a `Git::<op>` method in the git module holding the git side. Each of those methods had exactly one caller and one adapter, so the seam between them was hypothetical, and the CLI surface leaked across it anyway. We collapsed the split.

Each command is one module — a command slice — owning the command end to end: its clap args, its orchestration, its git arg construction, its report policy, and its rendering. The git operation lives inside its single caller as a private `impl Git` block: the method syntax is unchanged at the call site, but the compiler restricts it to the defining module. A slice depends only on the shared cores — git (runner seam, report engine, head resolution, output types), workspace (discovery, routing, run helpers), and render — never on another slice.

The governing rule: a helper lives with its single caller until a second caller earns its promotion to a core. Promotion is driven by callers, not speculation.

## Considered Options

- **`pub(crate)` operation methods called across slices** — rejected: it manufactures dependencies between slices that the compiler cannot police; private methods in the caller's module make the slices-never-depend-on-slices rule mechanical.
- **A central table of every operation's report policy** — rejected: it cuts across the slices and needs functions rather than constants anyway (`rm` picks its success policy from `--dry-run`).

## Consequences

- "What can `Git` do?" is not answerable from `git.rs` alone — the commands say what for. The core answers "run git and report outcomes".
- Cross-command policy differences (e.g. `fetch` reads stderr first, `pull` reads stdout first) are legitimate per-command facts pinned by each slice's tests — not drift to be centralized away.
