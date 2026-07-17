# Validate --src-prefix/--dst-prefix for combined diffs

Type: research
Status: open

## Question

Does `git diff --src-prefix=a/<repo>/ --dst-prefix=b/<repo>/` reliably produce VMR-root-relative paths for a combined monorepo-style diff, and where does it fall short? Verify against primary sources (git docs, real git behaviour):

- Rename and copy lines (`rename from`/`rename to`, `copy from`/`copy to`) — do they honour the prefixes?
- Binary diffs, mode-change-only entries, symlinks.
- User config interference: `diff.noprefix`, `diff.mnemonicPrefix`, `diff.srcPrefix`/`diff.dstPrefix` — do the command-line flags override them?
- Does colour (`--color=always`) change any header line in a way that matters?
- Is the concatenated multi-repo output a valid patch that `git apply` (from the VMR root, with `-p1`) would accept?
- Minimum git version for the flags involved.

Capture findings as a Markdown file on a throwaway `research/diff-prefix` branch; link it here.
