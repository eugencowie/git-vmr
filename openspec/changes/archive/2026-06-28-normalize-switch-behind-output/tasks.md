## 1. Switch Output Normalization

- [x] 1.1 Add a private helper in `src/git/switch.rs` that removes `by N commit(s)` from selected successful switch messages.
- [x] 1.2 Apply the helper only to successful `git vmr switch <branch>` output before creating the success outcome.

## 2. Tests

- [x] 2.1 Add integration coverage where two child repositories switch to the same behind tracking branch with different behind counts and render one grouped message.
- [x] 2.2 Cover that non-matching switch success output remains unchanged.

## 3. Verification

- [x] 3.1 Run `cargo test switch`.
- [x] 3.2 Run `mise exec -- cargo +nightly fmt --check`.
- [x] 3.3 Run `mise exec -- openspec validate normalize-switch-behind-output --strict`.
