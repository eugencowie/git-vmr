# Research: `--src-prefix`/`--dst-prefix` for combined VMR diffs

Ticket: [tickets/01-prefix-rewriting-research.md](../tickets/01-prefix-rewriting-research.md)

Question: does `git diff --src-prefix=a/<repo>/ --dst-prefix=b/<repo>/` reliably produce
VMR-root-relative paths for a combined monorepo-style diff, and is the concatenation a
valid patch for `git apply -p1` from the VMR root?

**Verdict: mostly yes, with one hard exception — rename/copy extended-header lines never
carry the prefix, so a combined patch containing rename/copy entries does NOT apply from
the VMR root. Generate per-repo diffs with `--no-renames` (and defensively `--no-color
--no-ext-diff --binary`) and the concatenated patch applies cleanly with `git apply -p1`.**

All behaviour below verified empirically with git 2.53.0 on macOS in scratch repos
(text edit, rename, copy, binary change, symlink retarget, mode-only change, new file,
two repos concatenated), plus primary-source docs as cited.

## What the prefixes do cover

With `--src-prefix=a/repoA/ --dst-prefix=b/repoA/`, the following lines all carry the
full `a/repoA/...` / `b/repoA/...` paths:

- `diff --git a/repoA/x b/repoA/x` header lines — for every entry type, including
  renames, copies, binary, mode-only, and symlink entries.
- `---` / `+++` lines.
- `Binary files a/repoA/x and b/repoA/x differ` lines.

Mode-only changes (`old mode`/`new mode`), symlink diffs, and binary entries otherwise
behave exactly like normal entries; nothing in them embeds an unprefixed path.

## The exception: rename/copy `from`/`to` lines

`rename from <path>` / `rename to <path>` / `copy from` / `copy to` lines print the path
**without any prefix** — this is by design, documented in the generated-patch format
(git-diff(1), "generating patch text with -p": extended header lines use `<path>` with
no `a/`/`b/`, and the rename example shows bare `rename from a` / `rename to b`).
Observed output:

```
diff --git a/repoA/willrename.txt b/repoA/renamed.txt
similarity index 100%
rename from willrename.txt
rename to renamed.txt
```

`git apply` uses those bare lines to locate the files, so from the VMR root `git apply
-p1` fails with `error: willrename.txt: No such file or directory`. Same for copies.
This is the one place where prefix rewriting is insufficient.

### Workaround (verified)

Generate each per-repo diff with `--no-renames`: renames become delete+add pairs (copies
become plain adds), every path is fully prefixed, and the concatenated multi-repo patch
applies cleanly from the VMR root with `git apply -p1 --index` — including binary content
(requires `--binary` on both diff and, for content, on the generating side), mode
changes, and symlinks. `git status` in the target even re-detects the rename (`R
repoA/willrename.txt -> repoA/renamed.txt`). Cost: rename similarity information is
lost from the displayed diff, and delete+add shows full file content for renamed files.
Alternative if display-only renames are wanted: show `-M` output but document that the
combined output is not `-p1`-applyable; or apply per-repo with `git apply
--directory=<repo>` instead of concatenating.

## User config interference (all verified)

Command-line flags win over every relevant config knob:

- `diff.noprefix=true` — overridden by explicit `--src-prefix`/`--dst-prefix`.
- `diff.mnemonicPrefix=true` — overridden.
- `diff.srcPrefix`/`diff.dstPrefix` (config added in git 2.45, per RelNotes/2.45.0) —
  overridden.
- `color.diff=always` **does** inject ANSI escapes even when output is piped; pass
  `--no-color` to be safe. Colour wraps whole lines in SGR codes (`ESC[1m...ESC[m`)
  without altering header text, but coloured output is not a valid patch.
- `diff.external` / `GIT_EXTERNAL_DIFF` replaces patch generation entirely; pass
  `--no-ext-diff` (verified: `-c diff.external=false` makes `git diff` fail unless
  `--no-ext-diff` is given).

Recommended invocation per repo:

```
git -c core.quotePath=false diff --no-color --no-ext-diff --no-renames --binary \
    --src-prefix=a/<repo>/ --dst-prefix=b/<repo>/ ...
```

(`core.quotePath` is a further display knob: non-ASCII paths get quoted+escaped
`"a/repoA/\346..."` headers by default; harmless to apply but affects display.)

Note the trailing slash on the prefix values is mandatory — git concatenates the prefix
and path verbatim (`--src-prefix=a/repoA` would yield `a/repoAfile.txt`).

## Colour

`--color=always` changes no header content, only wraps each line in SGR sequences. Fine
for terminal display; never feed coloured output to `git apply`. Generate the applyable
form with `--no-color` (or rely on git's own colouring for display and a separate
plumbing pass for patch output).

## Minimum git version

- `--src-prefix` / `--dst-prefix` / `--no-prefix`: git **1.5.4** (Feb 2008;
  RelNotes/1.5.4: `"a/" and "b/" e.g. "git diff --src-prefix=l/ --dst-prefix=k/"`).
- `--no-renames`: predates 1.5.4 as well.
- `--default-prefix` (not needed here): 2.41. `diff.srcPrefix`/`dstPrefix` config
  (which we must override): 2.45.

Any git a user could plausibly run satisfies the flag requirements; no version gate
needed beyond the project's existing baseline.

## Sources

- git-diff(1) man page (git 2.53.0), "generating patch text with -p" section — extended
  header format, unprefixed rename/copy paths.
- git release notes: `Documentation/RelNotes/1.5.4.adoc`, `2.45.0.adoc`
  (github.com/git/git).
- Empirical verification, git 2.53.0: scratch repos exercising rename (`-M`), copy
  (`-C --find-copies-harder`), binary, symlink, mode-only, config overrides, and
  `git apply -p1 --index` of a two-repo concatenated patch from a VMR root.
