## Context

Every design decision here was settled during the wayfinder effort
([map.md](map.md)); the closed tickets 01–03 hold the full rationale and
empirical verification (git 2.53.0), and [spec.md](spec.md) is the
consolidated contract. This document is the condensed technical view.

Current state: commands return a pure `Rendered { stdout, stderr }` and
printing happens once at the `emit` choke point ([render.rs](../../../src/render.rs));
git runs only through the Git runner seam (subprocess in production,
scripted fake in tests); user paths reach child repos via routing.

## Goals / Non-Goals

**Goals:**
- One combined, monorepo-style diff with VMR-root-relative paths.
- Applyable-patch contract when piped (`git apply -p1` from the VMR root).
- Git-like colour and paging without breaking the choke-point rule.

**Non-Goals:**
- Commit ranges (`A..B`) — revisions don't align across independent repos.
- Output-shape flags (`--stat`, `--name-only`, `-U<n>`, `-w`, …).
- Streaming child diffs through the pager (a future performance change
  behind the same emit seam).

## Decisions

1. **Prefixes via git itself** (`--src-prefix=a/<repo>/`,
   `--dst-prefix=b/<repo>/`): covers all header lines except
   rename/copy lines; CLI flags override all `diff.*` prefix config.
   `--no-ext-diff --binary` are passed defensively; explicit
   `--color=always|never` (resolved from one TTY fact) is always passed
   so `color.diff=always` cannot inject ANSI when piped. (Ticket 01.)

2. **Rename/copy headers fixed by a pure, diff-local transform** rather
   than `--no-renames`: prepend `<repo>/` to
   `rename from`/`rename to`/`copy from`/`copy to` within extended-header
   regions (from `diff --git` to the first `@@`, or the next `diff --git`
   / end for hunkless blocks). Quoted paths get the prefix inside the
   quotes; no re-escaping needed. Keeps rename detection on — the honest
   human view and the applyable patch are the same bytes. (Tickets 02/03,
   [research/quoted-paths.md](research/quoted-paths.md).)

3. **Paging is a property of emit, not the command**: `Rendered` gains
   `pager: Option<String>` (default `None` — every command keeps the
   identical shape, per the interface-symmetry preference). Diff decides
   *whether/with what* (TTY-gated, `--no-pager`, one `git var GIT_PAGER`
   from the VMR root through the runner seam); emit does the spawning
   (`sh -c`, `LESS=FRX`/`LV=-c` when unset), stays git- and TTY-free.
   Buffer, don't stream. Stderr prints after the pager exits; no re-print
   if the pager dies; direct-print fallback only if `sh` cannot spawn.
   Alternative rejected: a pager trait mirroring the Git runner — the
   pager command string itself is a sufficient seam. (Ticket 02.)

4. **Applyability is a contract, not an accident** (ticket 04): the
   integration suite round-trips `git apply -p1` onto a pristine VMR;
   breaking it is a breaking change. Cheap because colour — the only
   spoiler — is TTY-gated off exactly when the contract applies.

5. **Testing**: TTY is an injected bool on the CLI context
   (`for_tests` → `false`; no PTY harness). Combined stdout asserted
   with exact `assert_eq!` (deliberate break from the `contains`
   convention — byte fidelity is the product), two-sided with
   `fake.calls()` for child args. Emit pager tests use real `sh -c`
   recording scripts. Seven inline fixtures for the rewrite. (Ticket 03.)

## Risks / Trade-offs

- [Rewrite parses raw diff text] → region bounds are minimal and
  verified against real git output, including the hunkless-100%-rename
  and quoted-path cases; the apply round-trip is a whole-pipeline oracle.
- [Pager death loses the diff] → deliberate, git-faithful; documented.
- [`sh` spawn failure fallback is untested] → portable arrangement is
  impossible; kept as defensive code, explicitly not a contract.
- [Buffering large diffs in memory] → worktree-sized diffs are small;
  streaming remains possible behind the same seam if it ever matters.
