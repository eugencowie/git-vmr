# A VMR is a root repo with a declared manifest

A VMR was a plain directory outside Git whose child repos were discovered by scanning for subdirectories containing `.git`, so nothing about the VMR itself was versioned: membership could not be shared or cloned, and top-level files had no home. We made the VMR root a required Git repository — the root repo — which commits a `.gitvmr` manifest declaring each child repo by path and remote URL, and which takes part in every VMR operation, so a VMR behaves like a single Git repository. The manifest is authoritative and versions which repos belong, never which commits they are at: child repos stay independent.

## Considered Options

- **Submodules, or submodule-style commit pinning** — rejected: pinning brings detached heads, "modified content" noise, and two-phase commits, and contradicts the premise that child repos stay independent.
- **Keep the flat directory, with an optional root repo** — rejected: two root shapes double every path through routing, worktrees, and clone for little gain.
- **Keep discovery, with the manifest only recording URLs for clone** — rejected: two sources of truth for membership drift apart.
