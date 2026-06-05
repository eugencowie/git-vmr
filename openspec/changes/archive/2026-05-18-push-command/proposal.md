## Why

Users working across a virtual monorepo can fetch and pull across child repositories, but publishing local commits still requires entering each repository and running `git push` manually. A VMR-level push command reduces that repetition while preserving Git's per-repository push semantics and reporting all failures after every repository has been attempted.

## What Changes

- Add a `git vmr push [<repository> [<refspec>...]]` subcommand that attempts to push from every immediate child Git repository.
- Pass the optional repository and refspec arguments through to `git push` in each child repository without VMR-level remote, upstream, branch, authentication, or refspec validation.
- Treat push attempts as best-effort: a failure in one child repository SHALL NOT prevent push attempts in other discovered child repositories.
- Run child repository push attempts in parallel and report repository-specific results deterministically after all attempts complete.
- Succeed when all push attempts succeed, when there are no immediate child Git repositories, or when only non-Git child directories are present.
- Reuse existing VMR root discovery and global `-C <path>` behavior.

## Capabilities

### New Capabilities

- `push-command`: Defines `git vmr push` behavior for best-effort parallel pushes across immediate child Git repositories in a virtual monorepo.

### Modified Capabilities

## Impact

- Adds a new CLI subcommand and command module.
- Reuses immediate child repository discovery conventions established by aggregate commands such as `fetch`, `pull`, `merge`, `rebase`, `switch`, and `tag`.
- Shells out to Git for push semantics, including remote names, repository URLs, refspec handling, upstream selection, authentication, push rejection, hooks, and transport behavior.
- Adds integration tests for default pushes, repository arguments, refspec arguments, non-Git child skipping, no-child success, deterministic failure reporting, best-effort behavior, and effective working directory handling.
