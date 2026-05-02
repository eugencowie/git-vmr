## 1. Create the Rust project scaffold

- [ ] 1.1 Add `Cargo.toml` for the `git-vmr` Rust 2024 binary crate with project
  metadata, GPL-2.0-only licensing, and strict Rust and Clippy lint settings
- [ ] 1.2 Add the minimal `src/main.rs` executable entry point
- [ ] 1.3 Generate and commit `Cargo.lock`

## 2. Establish repository policy files

- [ ] 2.1 Expand `README.md` with the project purpose and GPL-2.0-only license
  notice
- [ ] 2.2 Add the GPL-2.0-only `LICENSE` file
- [ ] 2.3 Add cross-platform `.gitignore` rules for Rust output, local tool
  configuration, editor files, and operating-system artifacts
- [ ] 2.4 Add `AGENTS.md` with the format, lint, test, and `mise exec --`
  fallback commands

## 3. Configure local development tooling

- [ ] 3.1 Add `rustfmt.toml` with the repository's nightly formatting policy
- [ ] 3.2 Add `mise.toml` with Rust, Node, actionlint, and OpenSpec tools plus
  format, lint, test, and aggregate CI tasks
- [ ] 3.3 Generate and commit the `mise` lockfile

## 4. Add automated validation

- [ ] 4.1 Add a GitHub Actions workflow for pushes and pull requests
- [ ] 4.2 Configure CI to run nightly rustfmt checks, stable Clippy validation,
  and the Cargo test suite with Rust dependency caching

## 5. Enable spec-driven changes

- [ ] 5.1 Add the repository OpenSpec configuration with the `spec-driven`
  schema

## 6. Verify the project baseline

- [ ] 6.1 Run `cargo +nightly fmt --check`
- [ ] 6.2 Run `cargo clippy --all-targets`
- [ ] 6.3 Run `cargo test`
