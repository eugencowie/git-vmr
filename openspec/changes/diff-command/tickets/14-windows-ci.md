# Windows CI test job

Type: implementation
Blocked by: 12, 13

## Task

Add a `windows-latest` job to `.github/workflows/ci.yml` running the
existing test suite, so the Windows behaviour promised by
[spec.md](../spec.md) executes somewhere at all (today CI is
`ubuntu-latest` only; the release workflow builds the msvc target but
runs nothing).

Caveat to record in the workflow/job notes: GitHub's `windows-latest`
runners ship Git for Windows *with* `sh` on `PATH`, so the job does
**not** reproduce the hostile recommended-install environment (no `sh`
visible from cmd/PowerShell). It proves the honest-breakage class —
path separators, spawn semantics, `IsTerminal` assumptions, the VT
probe compiling and running — not the degradation paths, which stay
covered by the platform-neutral unit tests from ticket 12.
