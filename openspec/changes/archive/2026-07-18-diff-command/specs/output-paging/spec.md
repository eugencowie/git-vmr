## ADDED Requirements

### Requirement: Pager is an optional property of rendered output
`Rendered` SHALL carry `pager: Option<String>` — the shell command to page stdout through — defaulting to `None`, meaning print directly. Commands other than diff SHALL leave it `None` and their behaviour SHALL be unchanged.

#### Scenario: Default is direct printing
- **WHEN** a command returns `Rendered` with `pager: None`
- **THEN** emit SHALL print stdout and stderr directly, as today

### Requirement: Pager resolution is delegated to git
The diff command SHALL fill the pager field only when stdout is a TTY and `--no-pager` is absent, resolving the pager with one `git var GIT_PAGER` invocation through the Git runner seam, run from the VMR root. A resolution of `cat` or empty SHALL mean no paging. Emit SHALL remain git-free and TTY-free.

#### Scenario: Non-TTY means no pager
- **WHEN** stdout is not a TTY
- **THEN** the pager field SHALL be `None` and `git var GIT_PAGER` SHALL NOT be invoked

#### Scenario: --no-pager wins
- **WHEN** `git vmr diff --no-pager` runs with stdout a TTY
- **THEN** the pager field SHALL be `None`

### Requirement: Emit pages the buffered output
When the pager field is set, emit SHALL spawn the pager via `sh -c`, exporting `LESS=FRX` and `LV=-c` when unset, write the buffered stdout into the pager's stdin, wait for the pager to exit, then print stderr and propagate the command result for the exit code. The pager SHALL see only stdout; a pager that exits non-zero SHALL NOT cause the output to be re-printed. If `sh` itself cannot be spawned, emit SHALL fall back to printing directly.

#### Scenario: Pager receives the diff verbatim
- **WHEN** emit runs with a pager command that records its stdin
- **THEN** the recorded stdin SHALL equal the rendered stdout byte-for-byte

#### Scenario: Stderr lands after the pager exits
- **WHEN** emit pages output for a command that also has stderr text
- **THEN** the stderr text SHALL be printed only after the pager process has exited

#### Scenario: Dying pager loses the diff
- **WHEN** the pager exits non-zero before consuming its stdin
- **THEN** emit SHALL NOT print the rendered stdout directly
