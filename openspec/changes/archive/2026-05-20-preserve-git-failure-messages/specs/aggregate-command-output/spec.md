## ADDED Requirements

### Requirement: Fan-out failure output preserves selected Git messages
Fan-out Git command failure rendering SHALL preserve the selected Git message text without adding, removing, or normalizing a `fatal:` prefix. The renderer SHALL append repository attribution after the preserved message according to the existing aggregate repository suffix rules.

#### Scenario: Git fatal message is preserved
- **WHEN** a fan-out command fails in child repositories `alpha` and `zeta`
- **AND** Git emits `fatal: invalid reference: feature/auth` for both failures
- **THEN** stderr SHALL contain `fatal: invalid reference: feature/auth` exactly once
- **AND** stderr SHALL NOT contain `fatal: fatal: invalid reference: feature/auth`
- **AND** stderr SHALL identify both `alpha` and `zeta` as repositories that produced that failure

#### Scenario: Git non-fatal error prefix is preserved
- **WHEN** a fan-out command fails in child repository `backend`
- **AND** Git emits `error: branch 'feature/auth' not found`
- **THEN** stderr SHALL contain `error: branch 'feature/auth' not found (backend)`
- **AND** stderr SHALL NOT contain `fatal: error: branch 'feature/auth' not found (backend)`

## REMOVED Requirements

### Requirement: Fan-out failure output uses a single fatal prefix
**Reason**: Fan-out failure rendering now preserves the selected Git message exactly instead of applying a `fatal:` prefix policy.
**Migration**: Use `Fan-out failure output preserves selected Git messages`; callers and tests should assert the prefix Git emitted for the selected message.
