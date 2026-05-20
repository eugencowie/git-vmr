## 1. Rendering Boundary

- [x] 1.1 Use inline rendered CLI error text with explicit `fatal:` and `error:` prefixes.
- [x] 1.2 Change the top-level entry point to print returned errors verbatim instead of adding a blanket `fatal:` prefix.
- [x] 1.3 Replace aggregate downcast rendering with joined render-ready aggregate failure text from `print_results`.

## 2. Direct Error Classification

- [x] 2.1 Audit direct top-level error paths from working directory resolution, VMR discovery/routing, init, clone, mv, and command preflight logic.
- [x] 2.2 Convert environment, discovery, invocation, and unrecoverable setup failures to explicit `fatal:` errors.
- [x] 2.3 Convert selected tool-authored operand or validation failures to explicit `error:` errors where that better matches Git-style validation output.
- [x] 2.4 Ensure Git-emitted, hook-emitted, and fan-out aggregate failure messages remain verbatim and are not rewritten by direct-error prefixing.

## 3. Tests And Validation

- [x] 3.1 Update direct error tests that currently rely on top-level `fatal:` injection.
- [x] 3.2 Add coverage for a direct `error:` validation failure that must not be rewritten as `fatal: error:`.
- [x] 3.3 Keep or add coverage for explicit direct `fatal:` failures such as missing `-C` target and missing VMR root.
- [x] 3.4 Keep aggregate coverage for unprefixed and Git-prefixed fan-out failures without top-level prefix injection.
- [x] 3.5 Run the relevant Rust test suite and OpenSpec validation for `explicit-cli-error-prefixes`.
