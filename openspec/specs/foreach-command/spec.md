# foreach-command Specification

## Purpose
Define `git vmr foreach <command>` behavior for running shell commands across immediate child repositories in a virtual monorepo, including command capture, environment variables, parallel execution, buffered output, failure reporting, stdin handling, and existing VMR working directory behavior.

## Requirements

### Requirement: Foreach command runs shell commands across child repositories
The `git vmr foreach <command>` command SHALL discover the VMR root from the effective working directory, scan immediate child directories of the VMR root, skip non-Git child directories, and run the provided shell command in every immediate child Git repository.

#### Scenario: Run command in multiple repositories
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** user runs `git vmr foreach pwd`
- **THEN** the command SHALL run `pwd` in `backend`
- **AND** the command SHALL run `pwd` in `frontend`
- **AND** the command SHALL exit successfully when both child commands exit successfully

#### Scenario: Non-Git child directories are skipped
- **WHEN** a VMR contains child Git repository `backend` and non-Git directory `docs`
- **AND** user runs `git vmr foreach pwd`
- **THEN** the command SHALL run `pwd` in `backend`
- **AND** the command SHALL NOT run the command in `docs`
- **AND** the command SHALL NOT fail because of `docs`

#### Scenario: No child repositories found
- **WHEN** a VMR contains no immediate child Git repositories
- **AND** user runs `git vmr foreach pwd`
- **THEN** the command SHALL exit successfully
- **AND** stdout and stderr SHALL be empty

### Requirement: Foreach command captures trailing command arguments
The `git vmr foreach <command>` command SHALL require at least one command argument, SHALL capture one or more trailing command arguments, and SHALL evaluate the captured arguments as a single shell command string.

#### Scenario: Missing command is rejected
- **WHEN** user runs `git vmr foreach`
- **THEN** command parsing SHALL fail
- **AND** no child repository commands SHALL be attempted

#### Scenario: Multiple command words form one shell command
- **WHEN** a VMR contains child Git repository `backend`
- **AND** user runs `git vmr foreach echo hello`
- **THEN** the command SHALL evaluate `echo hello` in `backend`
- **AND** stdout SHALL contain `hello`

#### Scenario: Child command options are captured
- **WHEN** a VMR contains child Git repository `backend`
- **AND** user runs `git vmr foreach git status --short`
- **THEN** the command SHALL evaluate `git status --short` in `backend`
- **AND** `--short` SHALL be passed to the child shell command rather than parsed as a `git vmr foreach` option

#### Scenario: Shell features are evaluated
- **WHEN** a VMR contains child Git repository `backend`
- **AND** user runs `git vmr foreach 'echo $name && git rev-parse --show-toplevel'`
- **THEN** the command SHALL evaluate the command through the shell in `backend`
- **AND** stdout SHALL contain `backend`

### Requirement: Foreach command provides VMR environment variables
The `git vmr foreach <command>` command SHALL set VMR-specific environment variables for each child command.

#### Scenario: Environment variables describe child repository
- **WHEN** user runs `git vmr foreach 'printf "%s %s %s %s\n" "$name" "$sm_path" "$displaypath" "$toplevel"'` from a VMR root containing child repository `backend`
- **THEN** the child command SHALL receive `name` as `backend`
- **AND** the child command SHALL receive `sm_path` as `backend`
- **AND** the child command SHALL receive `displaypath` as `backend`
- **AND** the child command SHALL receive `toplevel` as the absolute VMR root path

#### Scenario: Display path respects effective working directory
- **WHEN** user runs `git vmr foreach 'echo "$displaypath"'` from inside child repository `frontend`
- **AND** the same VMR contains child repository `backend`
- **THEN** the child command for `backend` SHALL receive `displaypath` as the path from `frontend` to `backend`

#### Scenario: Sha1 is not synthesized
- **WHEN** user runs `git vmr foreach 'test -z "${sha1+x}"'`
- **THEN** the VMR command SHALL NOT set a `sha1` environment variable for child commands

### Requirement: Foreach command runs child commands in parallel
The `git vmr foreach <command>` command SHALL start child commands for discovered child Git repositories in parallel and SHALL wait for every started child command to complete before rendering final results.

#### Scenario: Later repositories run despite earlier failure
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** user runs `git vmr foreach 'test "$name" != backend'`
- **THEN** the command SHALL run the child command in `backend`
- **AND** the command SHALL run the child command in `frontend`
- **AND** the command SHALL exit with a non-zero status because the `backend` child command failed

#### Scenario: Parallel results render deterministically
- **WHEN** a VMR contains child Git repositories `alpha` and `zeta`
- **AND** the child command for `zeta` completes before the child command for `alpha`
- **AND** user runs `git vmr foreach 'echo "$name"'`
- **THEN** stdout SHALL render the `alpha` result before the `zeta` result

### Requirement: Foreach command renders buffered child output
The `git vmr foreach <command>` command SHALL buffer stdout and stderr from each child command and render child output after all child commands complete. Stdout output SHALL be rendered to stdout in deterministic repository order. Stderr output SHALL be rendered to stderr in deterministic repository order.

#### Scenario: Repository headers are shown by default
- **WHEN** a VMR contains child Git repository `backend`
- **AND** user runs `git vmr foreach 'echo ok'`
- **THEN** stdout SHALL contain `Entering 'backend'`
- **AND** stdout SHALL contain `ok`

#### Scenario: Quiet mode suppresses repository headers
- **WHEN** a VMR contains child Git repository `backend`
- **AND** user runs `git vmr foreach --quiet 'echo ok'`
- **THEN** stdout SHALL NOT contain `Entering 'backend'`
- **AND** stdout SHALL contain `ok`

#### Scenario: Child stderr is preserved on stderr
- **WHEN** a VMR contains child Git repository `backend`
- **AND** user runs `git vmr foreach 'echo problem >&2'`
- **THEN** stderr SHALL contain `problem`

#### Scenario: Empty successful child output still shows header
- **WHEN** a VMR contains child Git repository `backend`
- **AND** user runs `git vmr foreach true`
- **THEN** stdout SHALL contain `Entering 'backend'`
- **AND** the command SHALL exit successfully

### Requirement: Foreach command reports child command failures
The `git vmr foreach <command>` command SHALL exit with a non-zero status when one or more child commands exit with a non-zero status, and SHALL identify failed repositories and their exit statuses on stderr after rendering child command output.

#### Scenario: Single child failure is reported
- **WHEN** a VMR contains child Git repository `backend`
- **AND** user runs `git vmr foreach false`
- **THEN** the command SHALL exit with a non-zero status
- **AND** stderr SHALL identify `backend` as a failed repository

#### Scenario: Multiple child failures are reported deterministically
- **WHEN** a VMR contains child Git repositories `alpha` and `zeta`
- **AND** user runs `git vmr foreach false`
- **THEN** the command SHALL exit with a non-zero status
- **AND** stderr SHALL report the `alpha` failure before the `zeta` failure

#### Scenario: Successful repositories are not reported as failures
- **WHEN** a VMR contains child Git repositories `backend` and `frontend`
- **AND** user runs `git vmr foreach 'test "$name" = frontend'`
- **THEN** the command SHALL exit with a non-zero status
- **AND** stderr SHALL identify `backend` as a failed repository
- **AND** stderr SHALL NOT identify `frontend` as a failed repository

### Requirement: Foreach command closes child stdin
The `git vmr foreach <command>` command SHALL run child commands with empty stdin.

#### Scenario: Child command receives end of input
- **WHEN** a VMR contains child Git repository `backend`
- **AND** user runs `git vmr foreach 'cat'`
- **THEN** the child command SHALL receive end of input without reading from the parent process stdin
- **AND** the command SHALL complete without waiting for interactive input

### Requirement: Foreach command uses existing VMR working directory behavior
The `git vmr foreach <command>` command SHALL interpret the global `-C <path>` option the same way as existing commands and SHALL use the resolved working directory as the starting point for VMR root discovery.

#### Scenario: Running from inside a child repository
- **WHEN** user runs `git vmr foreach pwd` from inside child repository `frontend`
- **AND** a `.gitvmr/` marker exists in an ancestor directory
- **THEN** the command SHALL discover the ancestor VMR root
- **AND** the command SHALL run `pwd` across all immediate child Git repositories

#### Scenario: Running with -C
- **WHEN** user runs `git vmr -C /workspace/vmr/frontend foreach pwd`
- **AND** `/workspace/vmr/.gitvmr/` exists
- **THEN** the command SHALL discover `/workspace/vmr` as the VMR root
- **AND** the command SHALL run `pwd` across all immediate child Git repositories
