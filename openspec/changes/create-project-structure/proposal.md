## Why

The repository needs a consistent project foundation before feature development
can begin. Establishing the Rust crate, developer workflow, and automated checks
up front gives future changes a documented structure and a repeatable quality
baseline.

## What Changes

- Create an executable Rust crate for `git-vmr` with initial package metadata and
  strict compiler and Clippy lint settings.
- Add repository documentation, GPL-2.0-only licensing, and ignore rules for
  Rust build output, local tool configuration, editor files, and common
  operating-system artifacts.
- Define reproducible local tooling and format, lint, and test tasks with `mise`.
- Configure Rust formatting rules and document the repository's development
  checks for coding agents.
- Add a GitHub Actions workflow that runs formatting, linting, and tests for
  pushes and pull requests.
- Enable the spec-driven OpenSpec workflow for future repository changes.

## Capabilities

### New Capabilities

- `project-structure`: Provide the initial Rust project scaffold, repository
  metadata, local development workflow, CI quality checks, and spec-driven
  project configuration.

### Modified Capabilities

None.

## Impact

This change establishes the repository root layout, the initial `src/main.rs`
binary entry point, Cargo metadata and lockfile, local `mise` tool definitions,
Rust formatting policy, GitHub Actions CI, repository guidance, license and
README content, ignore rules, and OpenSpec configuration. It introduces the
baseline workflow that subsequent implementation changes must pass.
