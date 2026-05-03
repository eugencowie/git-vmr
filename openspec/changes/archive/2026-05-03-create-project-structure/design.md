## Context

The repository starts with only a minimal README and needs a foundation for
building `git-vmr` as a Git CLI wrapper. The initial structure must support Rust
development, make local checks easy to run, enforce the same checks in CI, and
document the licensing and contribution baseline before feature work expands
the codebase.

## Goals / Non-Goals

**Goals:**

- Establish an executable Rust crate with package metadata and a committed
  dependency lockfile.
- Define strict linting and formatting defaults so future code follows one
  consistent standard.
- Provide reproducible local tools and named quality-check tasks through
  `mise`.
- Enforce formatting, linting, and tests in GitHub Actions.
- Document the project purpose, license, agent-facing commands, generated-file
  exclusions, and spec-driven workflow.

**Non-Goals:**

- Implement the multi-repository Git wrapper behavior.
- Introduce runtime dependencies or a command-line argument framework.
- Configure packaging, release automation, or deployment.
- Define feature-specific OpenSpec requirements.

## Decisions

### Use a Rust 2024 executable crate

Create a binary crate named `git-vmr`, set the package version to `0.0.0`, and
commit `Cargo.lock`. The binary entry point will remain minimal until feature
work begins. A library-only crate was considered, but the project is intended
to ship a CLI and needs an executable entry point from the start.

### Fail on compiler and Clippy warnings

Configure Cargo lint tables to deny Rust warnings and all Clippy lints. This
makes the quality baseline explicit in package metadata and prevents new code
from accumulating avoidable lint debt. Allowing warnings during the bootstrap
would make later enforcement more disruptive.

### Separate local tool provisioning from CI execution

Use `mise.toml` to provision Rust, Node, `actionlint`, and OpenSpec for local
development, and expose `format`, `lint`, `test`, and aggregate `ci` tasks. Run
the corresponding Cargo commands directly in GitHub Actions so CI remains
small and uses focused Rust toolchain actions rather than requiring `mise`.

### Use nightly rustfmt with stable linting and tests

Run `cargo +nightly fmt` because the formatting policy opts into unstable
rustfmt features. Run Clippy and tests with the stable toolchain to keep normal
build validation aligned with stable Rust. A stable-only configuration was
considered, but it would require dropping the selected formatting controls.

### Establish repository policy files at the root

Add root-level README, GPL-2.0-only license text, cross-platform ignore rules,
agent instructions, rustfmt policy, and spec-driven OpenSpec configuration.
Keeping these files at conventional paths makes them discoverable by GitHub,
Cargo, editors, developer tools, and coding agents without extra setup.

## Risks / Trade-offs

- [Nightly rustfmt behavior changes over time] -> Keep formatting validation in
  CI so changes are detected immediately and can be handled intentionally.
- [Denying all Clippy lints can make future upgrades noisy] -> Treat toolchain
  updates as explicit maintenance work and resolve or narrowly allow lints when
  justified.
- [Using latest local tool versions can reduce reproducibility] -> Commit the
  `mise` lockfile and rely on CI as the authoritative quality gate.
- [The initial binary does not provide user-facing behavior] -> Keep the entry
  point intentionally minimal and add CLI behavior through subsequent specs.

## Migration Plan

1. Add the Rust crate scaffold and root repository policy files.
2. Add local tool definitions and quality-check tasks.
3. Add CI execution for format, lint, and test checks.
4. Add spec-driven OpenSpec configuration for future changes.
5. Run the complete quality-check workflow to verify the baseline.

Rollback consists of reverting the bootstrap files because no persisted data,
published API, or deployed runtime is introduced.

## Open Questions

None.
