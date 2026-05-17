## Why

`git vmr tag` can list local tags and create a lightweight tag across child repositories, but deleting a coordinated local tag still requires entering each child repository or scripting raw Git commands. Adding deletion closes the basic tag lifecycle for VMR workflows and matches the already-supported branch deletion pattern.

## What Changes

- Add `git vmr tag -d <tagname>` and `git vmr tag --delete <tagname>` to delete a local tag across immediate child Git repositories.
- Attempt deletion independently in every discovered child Git repository, skipping non-Git child directories through existing VMR repository discovery.
- Preserve Git's repository-local deletion behavior by delegating to `git tag -d <tagname>`.
- Report successful and failed deletion messages with repository suffixes in deterministic repository-name order.
- Keep tag listing and lightweight tag creation behavior unchanged.
- Continue rejecting unsupported tag operations such as force replacement, annotated or signed tags, explicit target objects, verification, filtering, patterns, and multi-tag deletion.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `tag-command`: Support deletion of a single local tag across child repositories through `-d`/`--delete`, while preserving existing list/create behavior and unsupported argument validation.

## Impact

- CLI parsing and dispatch in `src/cli.rs`.
- Aggregate tag command behavior in `src/cli/tag.rs`.
- Git tag command wrapper exports and execution in `src/git.rs` and `src/git/tag.rs`.
- Tag integration and parser tests in `tests/tag.rs` and `src/cli.rs`.
- Tag command documentation in `docs/git-vmr/tag.md`.
