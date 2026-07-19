# Quoted paths in rename/copy extended diff headers

Empirical research against real git. **Git version: 2.53.0** (macOS). Scratch repo under the session scratchpad; deleted after the experiment.

## Question

How does git format rename/copy extended headers when paths need C-style quoting
(spaces, double quotes, non-ASCII with `core.quotePath`)? How does that interact
with `--src-prefix`/`--dst-prefix` and `--color`? And does the git-vmr rewrite
transform — prepending `<repo>/` (no `a/`/`b/`) to `rename from`/`rename to`
lines — still work for quoted paths?

## Method

Created a repo with files `with space.txt`, `quote"file.txt`, `täst.txt`,
`copysrc.txt`; committed; `git mv` all three; staged a copy of `copysrc.txt`.
Ran `git diff --cached -M [-C]` with default prefixes, with
`--src-prefix=a/backend/ --dst-prefix=b/backend/`, with `--color=always`, and
with `core.quotePath=false`. Captured bytes via `cat -e` (`$` marks line end).
Then rewrote the real patch bytes (prepending `backend/`) and applied with
`git apply -p1` from a parent directory containing `backend/`.

## Observed bytes

### 1. `git diff --cached -M -C` (default prefixes)

```
diff --git a/with space.txt b/moved space.txt$
similarity index 100%$
rename from with space.txt$
rename to moved space.txt$
diff --git "a/quote\"file.txt" "b/newquote\"file.txt"$
similarity index 100%$
rename from "quote\"file.txt"$
rename to "newquote\"file.txt"$
diff --git "a/t\303\244st.txt" "b/n\303\266ved.txt"$
similarity index 100%$
rename from "t\303\244st.txt"$
rename to "n\303\266ved.txt"$
```

Key points:

- **Spaces alone do not trigger quoting.** `a/with space.txt` is emitted bare,
  on both the `diff --git` line and the rename lines.
- A double quote in the name triggers quoting; the quoted string uses C-style
  escapes (`\"`). Non-ASCII (with default `core.quotePath=true`) is emitted as
  octal escapes (`\303\244`) inside quotes.
- **`rename from`/`rename to` lines ARE quoted** when the path needs quoting —
  the historical claim that only the `diff --git` line is quoted is false in
  git 2.53. (Both sides are quoted independently: only the side that needs it.)
- Quoting wraps the **entire token including the `a/`/`b/` prefix** on the
  `diff --git` line: `"a/quote\"file.txt"`, not `a/"quote\"file.txt"`.

Copy case (only produced with `--find-copies-harder`; a plain staged `cp` shows
as `new file mode` under `-C`):

```
diff --git a/copysrc.txt b/copy dest.txt$
similarity index 100%$
copy from copysrc.txt$
copy to copy dest.txt$
```

Same shape as rename; space-only name again unquoted.

### 2. `--src-prefix=a/backend/ --dst-prefix=b/backend/`

```
diff --git a/backend/with space.txt b/backend/moved space.txt$
rename from with space.txt$
rename to moved space.txt$
diff --git "a/backend/quote\"file.txt" "b/backend/newquote\"file.txt"$
rename from "quote\"file.txt"$
rename to "newquote\"file.txt"$
diff --git "a/backend/t\303\244st.txt" "b/backend/n\303\266ved.txt"$
rename from "t\303\244st.txt"$
rename to "n\303\266ved.txt"$
```

- The prefix appears **inside the quotes** on the `diff --git` line
  (`"a/backend/quote\"file.txt"`).
- **`rename from`/`rename to` lines never carry the prefix** — they stay
  prefix-less regardless of `--src-prefix`/`--dst-prefix`.

### 3. `--color=always`

```
^[[1mdiff --git "a/quote\"file.txt" "b/newquote\"file.txt"^[[m$
^[[1mrename from "quote\"file.txt"^[[m$
^[[1mrename to "newquote\"file.txt"^[[m$
```

SGR codes sit strictly **outside the quotes**: one `ESC[1m` at line start, one
`ESC[m` at line end. No color codes ever appear inside or adjacent to the quoted
token; the quoted bytes are identical to the uncolored output.

### 4. `core.quotePath=false`

```
diff --git a/täst.txt b/növed.txt$          (bytes: tM-CM-$st.txt = UTF-8 raw)
rename from täst.txt$
rename to növed.txt$
```

Non-ASCII names are emitted as raw UTF-8, unquoted. Names containing `"` (or
control chars) are still quoted even with `quotePath=false` — quoting is only
suppressed for the "unusual byte" class, not for structurally ambiguous chars.

### 5. `git apply -p1` after the rewrite transform

Rewrote the real patch by prepending `backend/`:

- `diff --git` line: after `a/` / `b/`, i.e. **inside the quotes** when quoted:
  `"a/backend/quote\"file.txt"`.
- `rename from`/`rename to`: at the start of the path, **inside the quotes**
  when quoted: `rename from "backend/quote\"file.txt"`; bare paths become
  `rename from backend/with space.txt`.
- `---`/`+++` lines: after the `a/`/`b/` prefix.

Result: `git apply -p1` from the parent directory **applied cleanly**
(`APPLY_OK`); all renamed/created files landed under `backend/`, including
`newquote"file.txt` and `növed.txt`. Note `git apply -pN` strips `N-1`
components from `rename from`/`rename to` lines (they have no `a/`/`b/`), so
with `-p1` the prefix-less `backend/...` paths are used verbatim — exactly what
the transform produces.

## Conclusions

1. The rename-rewrite transform must handle quoted paths, and the rule is
   simple: **insert the repo prefix immediately after the opening `"` (if the
   path is quoted) or at the start of the path (if bare)**. Since the prefix
   itself (`backend/`) contains no characters needing escapes, no re-escaping
   of the rest of the string is required — the existing quoted bytes are kept
   verbatim.
2. Equivalently for the `diff --git` line: replace `a/` → `a/<repo>/` and
   `b/` → `b/<repo>/` whether or not a `"` precedes the `a`/`b`; git puts the
   prefix inside the quotes, so the transform must too.
3. `rename from`/`rename to`/`copy from`/`copy to` lines never carry
   `--src-prefix`/`--dst-prefix`, so the transform must not expect or add
   `a/`/`b/` there — plain `<repo>/` prepend is correct, and `git apply -p1`
   accepts it (verified).
4. Detection caveats: a space alone does not trigger quoting, so "line contains
   a space" is not a signal; and `core.quotePath=false` output can contain raw
   UTF-8 unquoted while `"`-containing names remain quoted. The transform must
   branch only on whether the path token starts with `"`.
5. `--color=always` wraps whole header lines in `ESC[1m ... ESC[m`; quotes and
   path bytes are unchanged inside. A transform operating on colored output
   must strip/skip the leading SGR before testing for `"`, but git-vmr rewrites
   uncolored plumbing output, so this is only relevant if coloring is applied
   pre-rewrite (it should not be).
