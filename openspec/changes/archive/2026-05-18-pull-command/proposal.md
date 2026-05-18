## Why

Users working across a virtual monorepo can fetch updates across child repositories, but integrating those updates still requires entering each repository and running `git pull` manually. A VMR-level pull command reduces that repetition while preserving Git's per-repository pull semantics and making failures visible after all repositories have been attempted.

## What Changes

- Add a `git vmr pull [<repository> [<refspec>...]]` subcommand that attempts to pull in every immediate child Git repository.
- Pass the optional repository and refspec arguments through to `git pull` in each child repository without VMR-level remote, upstream, branch, or refspec validation.
- Treat pull attempts as best-effort: a failure in one child repository SHALL NOT prevent pull attempts in other discovered child repositories.
- Run child repository pull attempts in parallel and report repository-specific results deterministically after all attempts complete.
- Succeed when all pull attempts succeed, when there are no immediate child Git repositories, or when only non-Git child directories are present.
- Reuse existing VMR root discovery and global `-C <path>` behavior.

## Capabilities

### New Capabilities

- `pull-command`: Defines `git vmr pull` behavior for best-effort parallel pulls across immediate child Git repositories in a virtual monorepo.

### Modified Capabilities

## Impact

- Adds a new CLI subcommand and command module.
- Reuses immediate child repository discovery conventions established by aggregate commands such as `fetch`, `merge`, `rebase`, `switch`, and `tag`.
- Shells out to Git for pull semantics, including remote names, repository URLs, refspec handling, upstream selection, merge or rebase behavior, authentication, conflicts, and transport behavior.
- Adds integration tests for default pulls, repository arguments, refspec arguments, non-Git child skipping, no-child success, deterministic failure reporting, best-effort behavior, and effective working directory handling.
