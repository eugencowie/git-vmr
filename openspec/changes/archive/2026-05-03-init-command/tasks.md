## 1. Dependencies and Project Setup

- [x] 1.1 Add `clap` (derive), `toml`, `anyhow`, `serde` (derive) to `Cargo.toml`
- [x] 1.2 Create module directory structure: `src/commands/` and `src/config/`

## 2. Config Types

- [x] 2.1 Create `src/config/core.rs` with `Core` struct (`version: u32`, serde Serialize/Deserialize)
- [x] 2.2 Create `src/config/config.rs` with `Config` struct (`core: Core`, serde Serialize/Deserialize)
- [x] 2.3 Create `src/config/mod.rs` with re-exports for `Config` and `Core`
- [x] 2.4 Add `Config::default()` that returns `Config { core: Core { version: 0 } }`

## 3. Init Command

- [x] 3.1 Create `src/commands/init.rs` with `init()` function that creates `.gitvmr/config`
- [x] 3.2 Implement idempotency: check if `.gitvmr/config` exists, silently return if so
- [x] 3.3 Create `.gitvmr/` directory if it doesn't exist, then write default config as TOML

## 4. CLI Wiring

- [x] 4.1 Create `src/commands/mod.rs` with clap `Command` enum and dispatch
- [x] 4.2 Rewrite `src/main.rs` with clap `#[derive(Parser)]`, binary name `git-vmr`, subcommand dispatch

## 5. Verification

- [x] 5.1 Build and verify `cargo build` succeeds
- [x] 5.2 Run `git vmr init` and verify `.gitvmr/config` is created with correct content
- [x] 5.3 Run `git vmr init` again and verify it succeeds without modifying the file
