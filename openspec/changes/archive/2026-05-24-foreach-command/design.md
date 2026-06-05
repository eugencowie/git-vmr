## Context

`git-vmr` operates on immediate child Git repositories under a VMR root marked by `.gitvmr/`. Existing aggregate commands discover the VMR root from the effective working directory, scan immediate child directories, skip non-Git children, and use deterministic repository ordering. Most write or network commands already run per-repository work in parallel and aggregate results.

`git submodule foreach` is the nearest user-facing analogue, but VMR repositories are independent child repositories rather than submodules recorded by a superproject index. That removes submodule-specific data such as the recorded gitlink commit while making performance more important for large VMRs.

## Goals / Non-Goals

**Goals:**
- Add `git vmr foreach <command>` as a convenient ad hoc command runner across immediate child Git repositories.
- Keep VMR discovery, repository inclusion, and ordering consistent with existing commands.
- Run child commands in parallel and buffer their output so final rendering remains deterministic.
- Preserve shell command semantics so users can write compound commands, variable expansion, redirection, and pipelines.
- Provide VMR-aware environment variables that cover the useful submodule-foreach analogues.

**Non-Goals:**
- Recursively discovering nested Git repositories.
- Filtering the repository set by pathspec or pattern.
- Emulating submodule-only data such as `$sha1` from a superproject gitlink.
- Streaming child command output live while commands are still running.
- Adding rollback, cancellation, or transaction semantics for commands that mutate repositories.

## Decisions

### Decision: Capture a trailing shell command

The CLI should accept one or more trailing command words and join them into a single shell command string. This supports both quoted commands such as `git vmr foreach 'echo $name'` and unquoted simple commands such as `git vmr foreach git status --short`. The command position should be trailing so options intended for the child command are captured after the command begins.

Alternative considered: require exactly one command string. That mirrors many shell examples, but it makes simple invocations unnecessarily awkward and differs from `git submodule foreach echo hello`, which accepts multiple words.

### Decision: Execute through the shell from each child repository root

Each child command should run with the child repository path as the process working directory. The command should be evaluated by the platform shell used for project-supported environments, so shell features such as variable expansion, `&&`, pipes, and redirection work as users expect from a `foreach` command.

Alternative considered: execute argv directly without a shell. That would avoid shell quoting concerns, but it would not be a close analogue to `git submodule foreach` and would make common multi-command usage much less useful.

### Decision: Run in parallel, render buffered outputs deterministically

The command should start one child process per discovered child repository in parallel, collect each process exit status, stdout, and stderr, then render results in repository-name order after all child processes complete. This keeps large VMRs fast while preserving deterministic final output.

Alternative considered: run sequentially and stop on first failure, matching `git submodule foreach`. That is familiar, but it gives up a major VMR performance advantage and hides later repository results.

### Decision: Treat failures as best-effort aggregate failures

A non-zero child command exit should not stop other child commands. After all child commands complete and outputs are rendered, the VMR command should exit non-zero if any child failed and identify the failed repositories and exit statuses on stderr.

Alternative considered: cancel outstanding commands after the first failure. That would be hard to make predictable, could leave partial child process trees running, and conflicts with the decision to use parallel fan-out.

### Decision: Replay stdout and stderr to their matching streams

For each repository, buffered stdout should be written to parent stdout and buffered stderr should be written to parent stderr. Repository entry headers should be written to stdout unless `--quiet` is set. Because stdout and stderr are separate streams, deterministic ordering is guaranteed within each stream, not as a single cross-stream transcript.

Alternative considered: merge stderr into stdout for exact block ordering. That would make terminal transcripts simpler, but it would break scripts that expect diagnostics on stderr.

### Decision: Provide VMR-specific environment variables

Child commands should receive:
- `name`: child repository name.
- `sm_path`: child repository path relative to the VMR root.
- `displaypath`: child repository path relative to the effective working directory.
- `toplevel`: absolute VMR root path.

The command should not set `sha1` because VMRs do not have a superproject index recording a child commit. Setting `sha1` to the child `HEAD` would be convenient but misleading.

Alternative considered: omit all compatibility variables. That would simplify implementation, but environment variables are one of the main reasons `git submodule foreach` is useful for scripts.

### Decision: Close child stdin

Child commands should receive empty stdin. Parallel commands sharing the parent's stdin can block unpredictably or race each other for input.

Alternative considered: inherit stdin. That supports interactive commands in a single repository, but `foreach` runs many repositories and this change prioritizes deterministic aggregate behavior.

## Risks / Trade-offs

- Buffered output can use significant memory for commands with very large output -> Accept this for v1 and document that output is collected before rendering.
- Shell evaluation can surprise users who expect argv-style execution -> Document that `<command>` is evaluated by the shell, matching the foreach analogue.
- Parallel mutation can create cross-repository side effects faster than users expect -> Keep the command explicit, do not add implicit rollback, and report all failures.
- Separate stdout/stderr streams cannot form one perfectly ordered transcript -> Preserve stream semantics and guarantee deterministic ordering within each stream.
- `--quiet` only suppresses entry headers, not child command output -> Match the useful part of the submodule analogue while keeping child command behavior visible.
