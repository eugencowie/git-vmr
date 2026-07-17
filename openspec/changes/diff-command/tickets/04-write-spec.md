# Write the spec and implementation tickets

Type: task
Status: open
Blocked by: 01, 02, 03

## Question

Produce the destination artifact: `openspec/changes/diff-command/spec.md` plus numbered implementation tickets, folding in the map's Notes constraints and the answers from [01-prefix-rewriting-research](01-prefix-rewriting-research.md), [02-pager-seam](02-pager-seam.md), and [03-testing-strategy](03-testing-strategy.md). Ready for a separate build effort; this map ends here.

Decide here (graduated from the map's fog by ticket 02's answer): does v1 *promise* an applyable patch when piped? The rename-line rewrite makes rename patches apply cleanly with `git apply -p1` from the VMR root, and colour — the only remaining spoiler — is TTY-gated off when piped, so the promise is cheap to make; decide whether to write it into the spec as a contract or leave it an unpromised property.
