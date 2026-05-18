## 1. CLI Surface

- [x] 1.1 Add a `Push` subcommand to the clap command enum with optional `repository` and trailing `refspecs` positional arguments.
- [x] 1.2 Add a new `src/cli/push.rs` module and dispatch `Command::Push` from `Cli::run`.
- [x] 1.3 Add parser unit tests for `git vmr push`, `git vmr push origin`, `git vmr push origin main`, multiple refspecs, and global `-C` handling.

## 2. Git Push Wrapper

- [x] 2.1 Add a new `src/git/push.rs` module and export it from `src/git.rs`.
- [x] 2.2 Build push arguments as `push` plus optional repository plus ordered refspecs.
- [x] 2.3 Invoke Git with the existing `git_output` helper so per-repository stdout, stderr, and exit status are captured.
- [x] 2.4 Return repository-suffixed success output from the first non-empty Git stderr line, falling back to stdout, and return no message when Git emits no output.
- [x] 2.5 Return repository-suffixed failure output from the first non-empty Git stderr line, falling back to stdout.

## 3. Aggregate Push Behavior

- [x] 3.1 Discover the VMR root from the effective working directory and scan immediate child Git repositories using existing `Vmr::repos`.
- [x] 3.2 Run child repository push attempts in parallel using the existing rayon aggregate command pattern.
- [x] 3.3 Preserve best-effort behavior by attempting every discovered repository even when one or more fail.
- [x] 3.4 Render successful messages to stdout and aggregate failures through the existing aggregate error path in deterministic repository-name order.
- [x] 3.5 Ensure no child repositories, only non-Git child directories, and pushes with no Git output produce successful empty output.
- [x] 3.6 Leave successful pushes, rejected pushes, and failed repositories exactly as produced by Git without rollback.

## 4. Integration Tests

- [x] 4.1 Add tests that default `git vmr push` publishes commits across multiple child repositories.
- [x] 4.2 Add tests for `git vmr push <repository>` and `git vmr push <repository> <refspec>...` argument forwarding.
- [x] 4.3 Add tests that a single positional argument is treated as the repository argument.
- [x] 4.4 Add tests that non-Git child directories are skipped and no child repositories succeed quietly.
- [x] 4.5 Add tests that one repository failure does not stop successful pushes in other repositories.
- [x] 4.6 Add tests for repository-suffixed success output, repository-suffixed failure output, and deterministic failure ordering.
- [x] 4.7 Add tests for missing upstream or default push destination delegation and rejected push reporting without rollback.
- [x] 4.8 Add tests for running from inside a child repository and running with global `-C`.

## 5. Documentation and Verification

- [x] 5.1 Add `push` command documentation under `docs/git-vmr/` and update the command index if required by the docs workflow.
- [x] 5.2 Update README command support documentation if it lists supported subcommands.
- [x] 5.3 Run formatting and the relevant unit and integration tests.
- [x] 5.4 Run `openspec validate push-command` and address any proposal/spec/task issues.
