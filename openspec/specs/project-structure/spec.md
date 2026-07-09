## Purpose

Define the baseline repository structure, tooling, policy files, and automated
validation required for development of the `git-vmr` executable.

## Requirements

### Requirement: Executable Rust crate scaffold
The repository SHALL define an executable Rust crate named `git-vmr` using the
Rust 2024 edition, include package metadata for the project, and commit the Cargo
dependency lockfile.

#### Scenario: Build the initial executable
- **WHEN** a developer runs `cargo build`
- **THEN** Cargo builds the `git-vmr` executable from `src/main.rs`

### Requirement: Strict Rust lint policy
The Rust crate SHALL deny compiler warnings and all Clippy lints through Cargo
package configuration.

#### Scenario: Validate the initial crate with Clippy
- **WHEN** a developer runs `cargo clippy --all-targets`
- **THEN** Clippy validates the project with warnings treated as errors

### Requirement: Repository documentation and licensing
The repository SHALL document the purpose of `git-vmr`, declare the
GPL-2.0-only license in Cargo metadata, include the corresponding license text,
and provide cross-platform ignore rules for generated and local-only files.

#### Scenario: Inspect repository metadata
- **WHEN** a developer reviews the repository root and Cargo package metadata
- **THEN** the project purpose, GPL-2.0-only license, license text, and ignore
  rules are available at conventional repository paths

### Requirement: Reproducible local development workflow
The repository SHALL configure local tools and named tasks for formatting,
linting, testing, and aggregate CI validation through `mise`.

#### Scenario: Run the aggregate local validation task
- **WHEN** a developer runs `mise run check`
- **THEN** `mise` runs the formatting check, Clippy validation, and test suite

### Requirement: Project formatting policy
The repository SHALL define a root rustfmt policy and SHALL validate formatting
with nightly rustfmt so the selected unstable formatting options are applied.

#### Scenario: Check source formatting
- **WHEN** a developer runs `cargo +nightly fmt --check`
- **THEN** rustfmt validates Rust source files against the repository policy

### Requirement: Automated quality checks
The repository SHALL run formatting, linting, and tests in GitHub Actions for
pushes and pull requests.

#### Scenario: Validate a pushed change
- **WHEN** a commit is pushed or a pull request is opened or updated
- **THEN** GitHub Actions runs `cargo +nightly fmt --check`,
  `cargo clippy --all-targets`, and `cargo test`

### Requirement: Agent-facing development guidance
The repository SHALL provide coding agents with the individual format, lint,
and test commands and SHALL document the `mise exec --` fallback for unavailable
commands.

#### Scenario: Discover repository checks
- **WHEN** a coding agent reads the repository guidance
- **THEN** the agent can identify the commands for formatting, linting, testing,
  and command fallback execution

### Requirement: Spec-driven change workflow
The repository SHALL configure OpenSpec to use the `spec-driven` schema for
future changes.

#### Scenario: Create a future OpenSpec change
- **WHEN** a developer creates an OpenSpec change in the repository
- **THEN** OpenSpec uses the `spec-driven` schema from the repository
  configuration
