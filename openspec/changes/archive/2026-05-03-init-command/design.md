## Context

git-vmr is a greenfield Rust project with no dependencies beyond the standard library. The codebase currently consists of a single `src/main.rs` with a hello-world program. This change introduces the first real functionality: the `init` command, which bootstraps a virtual monorepo workspace.

## Goals / Non-Goals

**Goals:**
- Establish CLI architecture using clap (derive) that scales to future commands
- Create the `.gitvmr/config` file with TOML serialization
- Set up the module structure (`commands/`, `config/`) that all future work follows
- Define config types (`Config`, `Core`) with serde for future extensibility
- Make `init` idempotent and safe to re-run

**Non-Goals:**
- Root detection (walking up directories to find `.gitvmr/`) — belongs to a future change
- Any commands beyond `init`
- Config validation beyond basic TOML structure
- Git repository integration or dependency

## Decisions

### 1. CLI Framework: clap with derive macros

**Choice**: `clap` v4 with `#[derive(Parser)]` and `#[derive(Subcommand)]`

**Rationale**: clap is the de facto standard for Rust CLIs. The derive macro provides type-safe argument parsing with minimal boilerplate. Since `git-vmr` is a git subcommand (git dispatches to `git-vmr` binary), we only need subcommand parsing, not a binary name.

**Alternatives considered**:
- `lexopt`: Minimal but too low-level for a growing CLI
- `argh`: Google-style, less ecosystem support

### 2. Module structure: one type per file

**Choice**:
```
src/
├── main.rs              ← CLI entry, clap app
├── commands/
│   ├── mod.rs           ← re-exports, Command enum dispatch
│   └── init.rs          ← init command logic
└── config/
    ├── mod.rs           ← re-exports
    ├── config.rs        ← Config struct
    └── core.rs          ← Core struct
```

**Rationale**: One type per file keeps modules focused and navigable as the project grows. Each command gets its own file. Config sections get their own files. `mod.rs` files handle re-exports.

### 3. Config serialization: serde + toml

**Choice**: `serde` (derive) for `Config` and `Core` types, serialized via the `toml` crate.

```rust
// config/core.rs
#[derive(Serialize, Deserialize)]
pub struct Core {
    pub version: u32,
}

// config/config.rs
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub core: Core,
}
```

**Rationale**: serde is the standard serialization framework in Rust. `toml` crate integrates with it directly. Using `Serialize` for writing and `Deserialize` for future reading. `version` as `u32` — simple integer for schema migration tracking.

### 4. Idempotency: file-level check

**Choice**: Check for `.gitvmr/config` specifically (not just `.gitvmr/` directory). If the file exists, exit silently with success. If the directory exists but not the file, create the file.

**Rationale**: Checking the file (not directory) is more precise. A partial state (directory without config) is recoverable. The silent-success behavior matches `git init` convention and makes `init` safe for scripts.

### 5. Error handling: anyhow

**Choice**: `anyhow` for error handling with `.context()` chaining.

**Rationale**: For a CLI tool where errors only flow toward `main`, `anyhow` keeps the code simpler than typed error enums. `.context()` attaches human-readable messages at call sites, and `{e:#}` in `main` renders the full chain.

## Risks / Trade-offs

- **[Risk] serde as transitive dependency** → serde is already pulled in by toml, so no extra cost. The derive macros are compile-time only.
- **[Risk] clap binary size** → clap adds to compile time and binary size. Acceptable for a CLI tool. Can be mitigated later with feature flags if needed.
- **[Trade-off] TOML vs custom format** → TOML is human-friendly and well-supported in Rust. Slightly more complex than a flat INI but much more extensible for future nested config sections.
