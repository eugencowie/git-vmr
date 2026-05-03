## Context

`git-vmr` is an early-stage Rust CLI with a single subcommand (`init`). The CLI uses clap derive. Currently, all subcommands receive the working directory via `std::env::current_dir()` called in `Cli::run()`. There is no way to override this.

The `init` subcommand already accepts `&Path` as its working directory parameter — it does not call `current_dir()` internally. This makes it straightforward to thread an alternate directory through without changing subcommand signatures.

## Goals / Non-Goals

**Goals:**
- Add a global `-C <path>` flag to `git-vmr` that sets the working directory for all subcommands
- Canonicalize the path to surface "directory does not exist" errors early
- Keep subcommands as pure functions of their inputs (no process-wide `set_current_dir`)

**Non-Goals:**
- Long form flag (`--working-dir`, `--directory`) — only `-C`
- Supporting relative path resolution against a base other than the actual working directory
- Changing how existing subcommands determine their working directory when `-C` is not provided

## Decisions

### Thread the directory explicitly rather than mutating process state

**Choice:** Resolve the directory in `Cli::run()` and pass it to subcommands, rather than calling `std::env::set_current_dir()`.

**Rationale:** Subcommands already accept `&Path` parameters. No process state mutation means tests remain straightforward and subcommand behavior is deterministic given its inputs. This matches the existing pattern.

**Alternative considered:** `set_current_dir` before dispatching — simpler but introduces hidden global state, makes tests fragile, and doesn't match the existing "pass root_dir explicitly" pattern.

### Canonicalize the path using `std::fs::canonicalize`

**Choice:** Use `canonicalize` to resolve the `-C` path to an absolute path, failing early if the directory does not exist.

**Rationale:** Catches user errors immediately. `git -C` uses `chdir` which also fails if the path doesn't exist. Canonicalize gives us the same guarantee without mutating process state.

**Trade-off:** `canonicalize` resolves symlinks and requires the path exist on disk. This is acceptable since the intent is to operate on a real directory.

## Risks / Trade-offs

- **`canonicalize` requires path to exist at parse time** → This is intentional. If the directory is created later by some other process, the user should run `git-vmr -C <path>` after it exists.
- **Future subcommands must accept `root_dir` parameter** → This is already the established pattern and a good practice.
