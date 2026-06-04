## Why

Users working in a virtual monorepo need a quick way to see which local tags exist across child repositories. Today `git vmr` lists branches and status across repositories, but tag inventory still requires running `git tag` in each child repository and comparing the results manually.

## What Changes

- Add a `git vmr tag` subcommand that lists local tags across all immediate child Git repositories in a VMR.
- Render output tag-centrically, grouping repositories by local tag name.
- Omit repository names when a tag exists in every discovered child repository.
- Show repository names when a tag exists in only a subset of discovered child repositories.
- Keep this change list-only: tag creation, deletion, verification, signing, annotation, filtering, and pattern matching are out of scope.

## Capabilities

### New Capabilities
- `tag-command`: Lists local tags across immediate child Git repositories in a virtual monorepo.

### Modified Capabilities

## Impact

- Adds a new CLI subcommand and command module.
- Reuses VMR root discovery and immediate-child repository scanning conventions established by `status` and `branch`.
- Shells out to Git for tag information, consistent with existing command modules.
- Adds integration and rendering tests for tag inventory output and repository-name elision.
