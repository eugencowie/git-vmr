# Write the spec and implementation tickets

Type: task
Status: resolved
Blocked by: 01, 02, 03

## Question

Produce the destination artifact: `openspec/changes/diff-command/spec.md` plus numbered implementation tickets, folding in the map's Notes constraints and the answers from [01-prefix-rewriting-research](01-prefix-rewriting-research.md), [02-pager-seam](02-pager-seam.md), and [03-testing-strategy](03-testing-strategy.md). Ready for a separate build effort; this map ends here.

Decide here (graduated from the map's fog by ticket 02's answer): does v1 *promise* an applyable patch when piped? The rename-line rewrite makes rename patches apply cleanly with `git apply -p1` from the VMR root, and colour — the only remaining spoiler — is TTY-gated off when piped, so the promise is cheap to make; decide whether to write it into the spec as a contract or leave it an unpromised property.

## Answer

Destination artifact produced: [spec.md](../spec.md) plus five
implementation tickets, ordered for building —
[05-cli-surface-and-tty](05-cli-surface-and-tty.md),
[06-combined-diff](06-combined-diff.md) (blocked by 05),
[07-rename-rewrite](07-rename-rewrite.md) (blocked by 06),
[08-pager](08-pager.md) (blocked by 05),
[09-integration-apply-round-trip](09-integration-apply-round-trip.md)
(blocked by 06, 07, 08). The spec folds in every constraint from the
map's Notes and the answers from tickets 01–03.

**Applyability decision: promised as a contract.** When stdout is not a
TTY, the combined output is a valid patch that `git apply -p1` from the
VMR root accepts — text, binary, mode-only, symlink, and rename changes.
Rationale: the rename rewrite already makes it hold and is verified
end-to-end, colour (the only spoiler) is TTY-gated off exactly when the
promise applies, and the integration round-trip test enforces it — so
the promise costs nothing while turning `git vmr diff > my.patch` into a
supported workflow. Breaking it is henceforth a breaking change.

The map ends here; building is a separate effort working tickets 05–09.
