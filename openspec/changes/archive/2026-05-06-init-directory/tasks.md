## 1. CLI Parsing

- [x] 1.1 Change the `Init` subcommand to accept an optional positional directory argument.
- [x] 1.2 Add parser coverage for `git-vmr init <directory>`.
- [x] 1.3 Add parser coverage showing global `-C <path>` can be combined with `init <directory>`.

## 2. Target Resolution

- [x] 2.1 Resolve a missing init directory argument to the effective working directory.
- [x] 2.2 Resolve a relative init directory argument against the effective working directory.
- [x] 2.3 Preserve absolute init directory arguments without joining them to the effective working directory.
- [x] 2.4 Return a clear error when the resolved init target exists and is not a directory.

## 3. Init Behavior

- [x] 3.1 Create the init target directory when it does not exist.
- [x] 3.2 Create `.gitvmr/config` inside an existing init target directory.
- [x] 3.3 Create `.gitvmr/config` inside a newly created init target directory.
- [x] 3.4 Preserve idempotent behavior when the target already contains `.gitvmr/config`.
- [x] 3.5 Preserve `-C` behavior so missing `-C` paths still fail before init target creation.

## 4. Verification

- [x] 4.1 Run the focused CLI and init unit tests.
- [x] 4.2 Run the full Rust test suite.
- [x] 4.3 Run OpenSpec validation for the `init-directory` change.
