## Why

Users working across a virtual monorepo often need to create the same release or coordination tag in every child repository. Today `git vmr tag` can show tag inventory across repositories, but creating a tag still requires entering each child repository and running `git tag <name>` manually.

## What Changes

- Add an optional `[tag-name]` positional argument to `git vmr tag`.
- Preserve current `git vmr tag` behavior when no tag name is provided: list local tags across child repositories.
- When a tag name is provided, attempt to create a lightweight tag at `HEAD` in every immediate child Git repository.
- Run tag creation independently per repository so one repository failure does not prevent attempts in other repositories.
- Report failed repository tag creations after all attempts complete, using concise Git-like error lines annotated with the repository name.
- Keep this change focused on lightweight tag creation: annotated tags, signed tags, forced replacement, explicit target objects, deletion, verification, filtering, formatting, and pattern matching remain out of scope.

## Capabilities

### Modified Capabilities

- `tag-command`: Add optional lightweight tag creation behavior and failure reporting semantics for `git vmr tag <tag-name>`.

## Impact

- CLI parsing for the `tag` subcommand.
- `src/cli/tag.rs` behavior for dispatching list mode versus create mode.
- `src/git/tag.rs` support for invoking Git tag creation.
- Tag command integration tests and CLI parsing tests.
- `tag-command` OpenSpec requirements and command documentation.
