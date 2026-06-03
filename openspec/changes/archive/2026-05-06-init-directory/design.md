## Context

`git vmr init` currently initializes `.gitvmr/config` in the effective working directory passed from the CLI layer. The effective working directory is either `std::env::current_dir()` or the canonicalized global `-C <path>` argument. The existing `-C` behavior intentionally requires the path to exist before any subcommand runs.

This change adds a separate optional target directory to the `init` subcommand. Unlike `-C`, the init target is part of init's creation behavior and can therefore be created when missing.

## Goals / Non-Goals

**Goals:**

- Support `git vmr init [directory]`.
- Preserve existing `git vmr init` behavior when no directory argument is supplied.
- Preserve existing `-C` behavior, including errors for missing or invalid `-C` paths.
- Resolve relative init directory arguments from the effective working directory.
- Create a missing init target directory before creating `.gitvmr/config`.
- Keep init idempotent for existing target directories and existing config files.

**Non-Goals:**

- Changing behavior for non-init subcommands.
- Allowing `-C` to create missing directories.
- Requiring the target directory to be inside a git repository.
- Adding new configuration fields or changing config file content.

## Decisions

1. Model the optional directory as an `Init { directory: Option<PathBuf> }` subcommand argument.

   Rationale: The behavior belongs specifically to `init`, not to all commands. Keeping it on the subcommand avoids weakening the existing global `-C` contract.

   Alternative considered: Reuse `-C` for this behavior. That would conflict with the current working-directory capability, where missing `-C` paths fail before subcommand dispatch.

2. Resolve relative init target paths against the effective working directory.

   Rationale: This matches how other path arguments are interpreted once `-C` has selected a working directory. It makes `git vmr -C base init project` initialize `base/project/.gitvmr`.

   Alternative considered: Resolve relative init target paths against the process current directory before applying `-C`. That would make init behave differently from other subcommand path handling and make `-C` less useful for scripting.

3. Let `init` create the target directory, then create `.gitvmr` inside it.

   Rationale: Directory creation is part of the new init capability. Creating the target before `.gitvmr` makes the command naturally support project bootstrapping.

   Alternative considered: Only create `.gitvmr` when the target exists. That would not satisfy the requested behavior and would remain too close to the existing `-C` semantics.

4. Error when the resolved init target exists and is not a directory.

   Rationale: Initializing inside a file path is nonsensical and would otherwise produce confusing lower-level filesystem errors.

   Alternative considered: Allow filesystem calls to fail naturally. Explicit validation gives a clearer user-facing error and a stable test target.

## Risks / Trade-offs

- Relative path resolution could be confused with `-C` path resolution -> Cover `git vmr -C base init project` with tests.
- Directory creation could accidentally mask an invalid parent path or permission problem -> Surface filesystem errors with context from the target path.
- Existing code and tests refer to `working_dir` as the init target -> Rename or introduce a distinct `target_dir` in the init implementation to avoid conceptual drift.
