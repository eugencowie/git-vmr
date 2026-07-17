# Issue tracker: Local Markdown

Tickets and specs for this repo live as Markdown files in `openspec/changes/`.

## Conventions

- One feature per directory: `openspec/changes/<change-slug>/`
- The specification is `openspec/changes/<change-slug>/spec.md`
- Implementation tickets are `openspec/changes/<change-slug>/tickets/<NN>-<slug>.md`, numbered from `01`
- Triage state is recorded as a `Status:` line near the top of each ticket file (see `triage-labels.md` for the role strings)
- Comments and conversation history append to the bottom of the file under a `## Comments` heading

## When a skill says "publish to the issue tracker"

Create a new file under `openspec/changes/<change-slug>/` (creating the directory if needed).

## When a skill says "fetch the relevant ticket"

Read the file at the referenced path. The user will normally pass the path or the ticket number directly.

## Wayfinding operations

Used by `/wayfinder`. The **map** is a file with one **child** file per ticket.

- **Map**: `openspec/changes/<effort>/map.md` — the Notes / Decisions-so-far / Fog body.
- **Child ticket**: `openspec/changes/<effort>/tickets/NN-<slug>.md`, numbered from `01`, with the question in the body. A `Type:` line records the ticket type (`research`/`prototype`/`grilling`/`task`); a `Status:` line records `claimed`/`resolved`.
- **Blocking**: a `Blocked by: NN, NN` line near the top. A ticket is unblocked when every ticket it lists is `resolved`.
- **Frontier**: scan `openspec/changes/<effort>/tickets/` for files that are open, unblocked, and unclaimed; first by number wins.
- **Claim**: set `Status: claimed` and save before any work.
- **Resolve**: append the answer under an `## Answer` heading, set `Status: resolved`, then append a context pointer (gist + link) to the map's Decisions-so-far in `map.md`.
