## Why

Users working across a virtual monorepo can inspect and mutate branches across child repositories, but updating remote-tracking refs still requires entering each repository and running Git manually. A VMR-level fetch command reduces that repetition while preserving Git's per-repository fetch semantics.

## What Changes

- Add a `git vmr fetch [<repository> [<refspec>...]]` subcommand that attempts to fetch in every immediate child Git repository.
- Pass the optional repository and refspec arguments through to `git fetch` in each child repository without VMR-level remote or ref validation.
- Treat fetch attempts as best-effort: a failure in one child repository SHALL NOT prevent fetch attempts in other discovered child repositories.
- Run child repository fetch attempts in parallel and report repository-specific results deterministically after all attempts complete.
- Succeed when all fetches succeed, when there are no immediate child Git repositories, or when only non-Git child directories are present.
- Reuse existing VMR root discovery and global `-C <path>` behavior.

## Capabilities

### New Capabilities

- `fetch-command`: Defines `git vmr fetch` behavior for best-effort parallel fetches across immediate child Git repositories in a virtual monorepo.

### Modified Capabilities

## Impact

- Adds a new CLI subcommand and command module.
- Reuses immediate child repository discovery conventions established by aggregate commands such as `branch`, `merge`, `rebase`, `switch`, and `tag`.
- Shells out to Git for fetch semantics, including remote names, repository URLs, refspec handling, authentication, and transport behavior.
- Adds integration tests for default fetches, repository arguments, refspec arguments, non-Git child skipping, no-child success, deterministic failure reporting, best-effort behavior, and effective working directory handling.
