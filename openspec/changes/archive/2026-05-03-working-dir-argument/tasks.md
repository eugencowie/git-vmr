## 1. CLI Struct Changes

- [x] 1.1 Add `working_dir: Option<PathBuf>` field to `Cli` struct with `#[arg(short = 'C')]` attribute
- [x] 1.2 Add `Canonicalize` error variant to the `Error` enum in `src/cli/mod.rs`

## 2. Working Directory Resolution

- [x] 2.1 Update `Cli::run()` to resolve `root_dir` from `-C` flag or `current_dir()`, with canonicalization for `-C`
- [x] 2.2 Surface canonicalization errors using the new `Canonicalize` error variant

## 3. Tests

- [x] 3.1 Test that `-C` with a valid directory passes the canonicalized path to the subcommand
- [x] 3.2 Test that `-C` with a non-existent path returns an error
- [x] 3.3 Test that omitting `-C` falls back to `current_dir()`
