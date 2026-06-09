## Context

git-vmr currently has repository-local configuration stored in `.gitvmr/config`, created by `git vmr init` and used as VMR metadata. Future features need configuration that applies across all virtual monorepos for a user, so the CLI needs a separate global configuration path and loading mechanism.

The global file must not change the meaning or lifecycle of `.gitvmr/config`. Commands should continue to resolve their effective working directory from `-C` or the process current directory, while shared startup state should move behind a CLI context object.

## Goals / Non-Goals

**Goals:**
- Introduce a user-level `git-vmr/config.toml` global config file under the platform config directory.
- Support `GIT_VMR_CONFIG_DIR` for tests and controlled environments.
- Fail fast when an existing global config file cannot be parsed.
- Treat a missing global config file as default configuration.
- Route command dispatch through a shared CLI context that carries the invoked command name, effective working directory, and global config access.

**Non-Goals:**
- The change will not add user-facing config commands.
- The change will not migrate, rewrite, or replace `.gitvmr/config`.
- The change will not introduce feature-specific global settings beyond the existing core version shape.
- The change will not create the global config file automatically when it is missing.

## Decisions

1. Resolve the global config path with a platform directory helper.

   The implementation will use a standard platform config directory dependency to locate the user config root, then append `git-vmr/config.toml`. This keeps behavior aligned with OS conventions instead of hard-coding home-directory paths. `GIT_VMR_CONFIG_DIR` will override the config root so tests can avoid reading a developer's real config.

   Alternative considered: store global config beside `.gitvmr/config`. That would not support settings shared across virtual monorepos and would blur repository-local metadata with user-level preferences.

2. Parse missing and malformed files differently.

   A missing global config file will load as defaults so existing users can run commands without creating new files. A malformed file will return an error before the subcommand runs so users see configuration problems directly and commands do not run under partially understood settings.

   Alternative considered: ignore parse failures and continue with defaults. That would make invalid user configuration difficult to diagnose and could hide future settings that materially affect command behavior.

3. Keep local and global config types separate while sharing core schema pieces.

   The repository-local `Config` and user-level `GlobalConfig` types will remain distinct, but they can share nested config structs such as `Core`. This preserves the current `.gitvmr/config` contract while allowing global config to grow independently.

   Alternative considered: reuse one top-level config type for both files. That would couple local VMR metadata and user preferences, making future additions harder to reason about.

4. Introduce `CliContext` for command dispatch.

   Commands will receive a shared CLI context rather than separate `bin_name` and `working_dir` arguments. Initially, commands can read the same values they already use, while future work can access global config without widening every command signature again.

   Alternative considered: pass global config as a third argument to command dispatch. That would address the immediate need but would keep growing the command boundary as more shared runtime state is added.

## Risks / Trade-offs

- User has a malformed global config file → Fail fast with a parse error before running any subcommand, and cover the behavior with an integration test.
- Platform config directory cannot be resolved → Return a startup error that identifies the config directory resolution failure.
- Tests accidentally read a developer's real config → Centralize integration test command creation so every test command sets `GIT_VMR_CONFIG_DIR` to an isolated temporary root.
- New dependency expands the lockfile → Keep the dependency limited to platform config path resolution and avoid pulling in broader configuration frameworks.
