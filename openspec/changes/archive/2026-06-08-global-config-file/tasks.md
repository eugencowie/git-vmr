## 1. Configuration Model

- [ ] 1.1 Add a platform config directory dependency for resolving user-level configuration paths.
- [ ] 1.2 Extract the shared `Core` configuration section into a reusable config module.
- [ ] 1.3 Add a `GlobalConfig` type with default values and TOML serialization/deserialization coverage.
- [ ] 1.4 Implement global config path resolution using `GIT_VMR_CONFIG_DIR` first, then the platform config directory.
- [ ] 1.5 Implement global config loading that returns defaults for missing files and errors for malformed files.

## 2. CLI Context

- [ ] 2.1 Add a `CliContext` type that stores the display command name, effective working directory, and lazy global config access.
- [ ] 2.2 Update CLI startup to build `CliContext` after resolving `-C`.
- [ ] 2.3 Load global config before dispatching the requested command so malformed config fails before subcommand execution.
- [ ] 2.4 Update command dispatch to receive `CliContext` instead of separate command name and working directory arguments.
- [ ] 2.5 Preserve existing command behavior by reading the display command name and working directory from `CliContext`.

## 3. Integration Test Isolation

- [ ] 3.1 Centralize integration test command construction in a shared test helper module.
- [ ] 3.2 Set `GIT_VMR_CONFIG_DIR` to an isolated temp directory for every integration test command.
- [ ] 3.3 Write a minimal default global config into the isolated config directory for integration tests.
- [ ] 3.4 Ensure the shared helper is not compiled as a standalone zero-test integration target.

## 4. Verification

- [ ] 4.1 Add tests for global config path resolution with and without `GIT_VMR_CONFIG_DIR`.
- [ ] 4.2 Add tests for missing global config loading as defaults.
- [ ] 4.3 Add tests for malformed global config load failures.
- [ ] 4.4 Add an integration test that malformed global config fails before VMR root discovery.
- [ ] 4.5 Run formatting, unit tests, integration tests, and clippy with warnings denied.
