# Where does the pager live?

Type: grilling
Status: resolved

## Question

Where does pager support sit in the codebase — a diff-local detail inside the command, or a render-layer facility other commands could later use? Decide:

- How paging composes with the existing rule that printing happens once at a single choke point (see CONTEXT.md "Rendering").
- Streaming child diffs through the pager as they arrive vs buffering the combined diff and paging once.
- How repo-outcome failure reporting interleaves with paged output: stderr vs after the pager exits, and what the pager sees.
- How pager resolution (`GIT_PAGER` → `core.pager` → `PAGER` → `less -FRX`) is tested/faked.

Consult `/codebase-design` for the seam vocabulary. The user prefers interface symmetry — mirrored type shapes over avoiding hypothetical seams.

## Answer

Grilled to shared understanding; six decisions:

1. **Paging is a property of the emit choke point, not the diff command.**
   `Rendered` gains one field: `pager: Option<String>` — the shell command to
   page stdout through, `None` (the default) meaning print directly. Every
   command keeps the identical mirrored shape; only diff fills the field.
   No render-layer restructuring is needed.

2. **Buffer, don't stream.** Commands stay pure and return the finished
   combined diff; emit pages it once. Streaming would hand commands a live
   writer and dissolve the choke point, to save milliseconds on
   worktree-sized diffs. If it ever matters, streaming is a performance
   change behind the same emit seam, not a spec change.

3. **The pager sees only the diff (stdout); failures land on stderr after
   the pager exits.** Emit's order: spawn pager → write buffered stdout into
   its stdin → wait → print `rendered.stderr` → propagate the failure for a
   non-zero exit. This is today's print order with a `wait()` in between,
   and matches git (exit deferred until the pager closes).

4. **Resolution is delegated to git**: one `git var GIT_PAGER` invocation
   through the existing Git runner seam, run from the VMR root (global
   config only — a child repo's local `core.pager` cannot hijack the
   combined view). Verified: it resolves the full
   `GIT_PAGER` → `core.pager` → `PAGER` → default chain, returns the real
   pager even when its own stdout is piped, and works outside a repository.
   Mirror git's spawn behaviour: launch via `sh -c`, export `LESS=FRX` and
   `LV=-c` when unset.

5. **Division of labour**: the diff command decides *whether* and *with
   what* — one injected TTY fact (carried on the CLI context) gates both
   `--color` and paging so they can never disagree; `--no-pager` and
   non-TTY leave the field `None`. Emit does the spawning, stays git-free
   and TTY-free, and falls back to printing directly if the spawn fails.
   Testing needs no new trait: resolution is scripted-fake territory
   (`var GIT_PAGER`), TTY is a bool set directly in tests, and the pager
   string itself is the seam — emit tests pass `cat` or a recording script.

6. **Rename/copy lines are fixed by a diff-local pure transform** (follow-up
   to [01-prefix-rewriting-research](01-prefix-rewriting-research.md)):
   per child, before concatenation, prepend `<repo>/` (no `a/`/`b/`) to
   `rename from`/`rename to`/`copy from`/`copy to` lines within the
   extended-header region (between `diff --git` and the first `@@`).
   Verified empirically: the rewritten combined patch applies cleanly with
   `git apply -p1` from the VMR root (unrewritten, apply rejects it as
   "inconsistent old filename"); coloured headers are a single SGR wrap
   around unchanged text, so the same transform covers the TTY view.
   This dissolves the `--no-renames` workaround — rename detection stays
   on everywhere, and piped output is both the honest human view and an
   applyable patch. Colour remains the only applyability spoiler, and it
   is already TTY-gated off when piped. Whether the spec *promises*
   applyability is [04-write-spec](04-write-spec.md)'s call.

Emit knows nothing about git or patch formats; the render layer stays
format-blind; `Rendered` still carries only finished text plus where to
send it.
