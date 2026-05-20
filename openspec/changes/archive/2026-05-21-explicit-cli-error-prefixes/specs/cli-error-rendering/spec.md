## ADDED Requirements

### Requirement: Direct CLI errors render with explicit prefixes
Direct non-aggregate `git-vmr` errors SHALL render exactly as produced by the command or code path that creates the user-facing error text. The top-level entry point SHALL NOT add a blanket `fatal:` prefix to errors that reach it.

#### Scenario: Fatal working directory error is explicit
- **WHEN** user runs `git vmr -C missing status`
- **AND** `missing` cannot be resolved as the effective working directory
- **THEN** stderr SHALL contain a direct error beginning with `fatal: cannot change to 'missing'`
- **AND** stderr SHALL NOT contain `fatal: fatal: cannot change to 'missing'`

#### Scenario: Fatal VMR discovery error is explicit
- **WHEN** user runs `git vmr status` outside a virtual monorepo
- **THEN** stderr SHALL contain `fatal: not a virtual monorepo (or any of the parent directories): .gitvmr`
- **AND** stderr SHALL NOT contain `fatal: fatal: not a virtual monorepo`

### Requirement: Tool-authored validation errors can use non-fatal prefixes
Tool-authored direct validation failures SHALL choose their user-facing prefix at the error creation or command boundary. Validation failures that are not classified as fatal SHALL render with their selected prefix and SHALL NOT be rewritten by the top-level renderer.

#### Scenario: Direct validation error keeps error prefix
- **WHEN** user runs a command whose own operand validation fails after command dispatch
- **AND** the command renders that failure as `error: destination 'frontend/app.rs' already exists`
- **THEN** stderr SHALL contain `error: destination 'frontend/app.rs' already exists`
- **AND** stderr SHALL NOT contain `fatal: error: destination 'frontend/app.rs' already exists`

### Requirement: Aggregate failure text remains verbatim at top level
Fan-out aggregate failure rendering SHALL pass render-ready failure lines to the top-level error path without adding, removing, or normalizing prefixes. Joined aggregate errors SHALL preserve each rendered line exactly.

#### Scenario: Aggregate unprefixed failures stay unprefixed
- **WHEN** a fan-out command fails in child repositories `alpha` and `zeta`
- **AND** the selected failure messages are `alpha rejected` and `zeta rejected`
- **THEN** stderr SHALL contain `alpha rejected (alpha)`
- **AND** stderr SHALL contain `zeta rejected (zeta)`
- **AND** stderr SHALL NOT contain `fatal: alpha rejected (alpha)`
- **AND** stderr SHALL NOT contain `fatal: zeta rejected (zeta)`

#### Scenario: Aggregate Git prefixes remain unchanged
- **WHEN** a fan-out command fails in child repository `backend`
- **AND** Git emits `error: branch 'feature/auth' not found`
- **THEN** stderr SHALL contain `error: branch 'feature/auth' not found (backend)`
- **AND** stderr SHALL NOT contain `fatal: error: branch 'feature/auth' not found (backend)`
