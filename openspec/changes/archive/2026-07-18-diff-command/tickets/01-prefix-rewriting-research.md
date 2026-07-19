# Validate --src-prefix/--dst-prefix for combined diffs

Type: research
Status: resolved

## Question

Does `git diff --src-prefix=a/<repo>/ --dst-prefix=b/<repo>/` reliably produce VMR-root-relative paths for a combined monorepo-style diff, and where does it fall short? Verify against primary sources (git docs, real git behaviour):

- Rename and copy lines (`rename from`/`rename to`, `copy from`/`copy to`) — do they honour the prefixes?
- Binary diffs, mode-change-only entries, symlinks.
- User config interference: `diff.noprefix`, `diff.mnemonicPrefix`, `diff.srcPrefix`/`diff.dstPrefix` — do the command-line flags override them?
- Does colour (`--color=always`) change any header line in a way that matters?
- Is the concatenated multi-repo output a valid patch that `git apply` (from the VMR root, with `-p1`) would accept?
- Minimum git version for the flags involved.

Capture findings as a Markdown file on a throwaway `research/diff-prefix` branch; link it here.

## Answer

Full findings: `openspec/changes/archive/2026-07-18-diff-command/research/diff-prefix.md`. Verified empirically with git 2.53.0 plus git-diff(1) and git release notes.

- The prefixes cover `diff --git`, `---`/`+++`, and `Binary files ... differ` lines for every entry type (text, binary, mode-only, symlink). **Exception:** `rename from`/`rename to` and `copy from`/`copy to` lines are unprefixed by design (documented patch format), so a combined patch containing renames/copies fails `git apply -p1` from the VMR root (`error: <old-path>: No such file or directory`).
- Workaround (verified): generate per-repo diffs with `--no-renames --binary`; the concatenated multi-repo patch then applies cleanly with `git apply -p1 --index` from the VMR root, including binary, mode, and symlink changes. Cost: renames render as delete+add.
- CLI flags override `diff.noprefix`, `diff.mnemonicPrefix`, and `diff.srcPrefix`/`diff.dstPrefix` (all verified). But `color.diff=always` injects ANSI even when piped and `diff.external` replaces patch output — pass `--no-color --no-ext-diff` defensively. Trailing slash on the prefix value is mandatory (verbatim concatenation).
- `--color=always` only wraps lines in SGR codes; header text is unchanged, but coloured output is not applyable.
- Minimum git: `--src-prefix`/`--dst-prefix` since git 1.5.4 (2008) — no version gate needed.
