# Root repo with a declared manifest

## Why

A VMR is a plain directory outside Git whose child repos are discovered by scanning, so nothing about the VMR itself is versioned. Membership cannot be shared or cloned, top-level files (READMEs, CI, tooling) have no home, and worktree roots need a bespoke marker lifecycle. Making the VMR root a Git repository that declares its child repos — operated on as if the whole were a single Git repository — fixes all three at once.

## What Changes

- **BREAKING** The VMR root is a Git repository, the *root repo*, and is required. It commits a `.gitvmr` *manifest* declaring each child repo by name, path, and remote URL — no branch, no commit pinning. Relative URLs resolve against the root repo's `origin`, as submodule URLs do. See ADR `docs/adr/0004-root-repo-with-declared-manifest.md`.
- **BREAKING** Membership is declared, not discovered. The manifest is read from the working tree and is authoritative: undeclared repos on disk are not part of the VMR; declared repos missing from disk produce a warning, not an error. Child paths stay flat (a single path component); nested paths are out of scope.
- **BREAKING** The root repo is a *member repo* alongside the child repos and takes part in every VMR operation, with no opt-out. It is labelled `.` in output and runs first, followed by child repos sorted by path. `foreach` runs in it too, with `name` and `sm_path` set to `.`.
- **BREAKING** Routing: paths under the VMR root that lie outside every child repo route to the root repo, including `.gitvmr` itself (no longer rejected as metadata). The aggregate path expands to every member repo.
- Each child repo is ignored by an anchored entry (`/backend/`) in the root repo's committed `.gitignore`. git-vmr writes these entries only when it writes the manifest itself; when a declared child is not ignored, it warns on workspace open instead of editing files.
- Membership changes are ordinary root-repo commits, made by *declaring* and *undeclaring*:
  - `git vmr add` declares undeclared, unignored repos at the top level of the VMR root — explicitly (`add x`, `add x/file`) or implicitly when its scope covers them (`add -A`) — and then stages as usual over the resulting membership.
  - `git vmr rm --cached <child root>` undeclares a child, leaving its directory on disk. **BREAKING** plain `rm` on a child root is refused with a hint (today `rm -r backend` deletes every tracked file in `backend`).
- **BREAKING** `mv` refuses a child root as source or destination; renaming child repos is a follow-up.
- `git vmr init` runs `git init` if needed, writes the manifest, and backfills it from Git repositories already in the directory using each one's `origin` (repos without one are skipped with a reason). This is the upgrade path for existing VMRs. Committing is left to the user.
- **BREAKING** `git vmr clone <url>` clones the root repo and then every declared child repo, like `git clone --recurse-submodules`. `git vmr clone` with no URL, inside a VMR, clones declared-but-missing child repos (e.g. after a pull adds one); `pull` never clones silently.
- **BREAKING** Worktree roots are linked worktrees of the root repo, each governed by its own checked-out manifest. The separate marker file disappears.
- Discovery walks up from the working dir to the nearest root-repo checkout carrying a `.gitvmr` manifest; nearest wins, so VMRs do not compose. Old-style VMRs and worktree roots, and a root repo on a branch without a manifest, fail with clear errors pointing to the fix.
- **BREAKING** VMR config (`.gitvmr/config`) is deleted; it was written but never read.

## Affected Commands

- Discovery (every workspace command): finds a root-repo checkout carrying a manifest; clear errors for old-style VMRs and for branches without a manifest.
- `init`: creates the root repo and manifest and backfills from existing repos, replacing `.gitvmr/config` creation.
- `clone`: builds the whole VMR instead of delegating to `git clone`; with no URL, clones missing children.
- `add`, `rm`: declaring and undeclaring; the root repo is a routing target.
- `mv`: refuses child roots.
- `foreach`: runs in the root repo, with the member environment.
- `worktree`: worktree roots are root-repo worktrees; the marker lifecycle is replaced by the root repo's worktree lifecycle.
- Output: member repo labels (`.` for the root repo), ordering, and aggregation scope.
- Every other workspace command (`status`, `commit`, `branch`, `switch`, `tag`, `merge`, `rebase`, `reset`, `restore`, `diff`, `fetch`, `pull`, `push`) fans out over member repos rather than child repos only.

## Design

Delivered as a stack of PRs, each ending in a verified state, each updating the README and `docs/parity/` pages it affects, and titled with `!` where it breaks behavior so release-plz records it. Worktree commands are broken between steps 1 and 5; nothing is released mid-stack, so no compatibility code for the old marker is written.

### Step 1 — Manifest, discovery, init

The root repo is not yet a member; children come from the manifest instead of scanning.

- **Format.** TOML, named as close to `.gitmodules` as possible, with no version key in this first version (absence means v1):

  ```toml
  [repo.libfoo]
  path = "foo"
  url = "git://foo.com/git/lib.git"
  ```

  The name identifies the entry; the path is where the repo lives. Output labels, routing, worktree layout, and `.gitignore` lines use the path; the name appears only as the manifest key and as foreach's `$name`.
- **Validation** is strict, at the parse boundary, with errors pointing at the line: `path` and `url` required and non-empty; `path` a single component, not `.`, `..`, `.git`, or `.gitvmr`; no duplicate paths; unknown keys are errors (a typo like `ulr` must not pass). A parse failure is fatal for every workspace command.
- **Reading and writing.** Read with `toml` + serde (`deny_unknown_fields`, span-carrying errors). Write with a new `toml_edit` dependency so comments and formatting survive edits, used only by the declare/undeclare code — outside `FileStore`, whose full re-serialization suits config and state but not a hand-edited, committed file.
- **Discovery** is a filesystem check, no git subprocess: walking up from the working dir, a directory is a VMR root when `.gitvmr` is a file and `.git` exists beside it (directory or file, covering worktrees). Errors:
  - `.gitvmr` is a directory → old-style VMR; run `git vmr init` to convert.
  - `.gitvmr` is an empty file with no `.git` → old-style worktree root; remove its child worktrees with `git worktree remove` in each repo, then recreate it with `git vmr worktree add`. No migration command.
  - No match → "not a virtual monorepo", hinting that the root repo may be on a branch without a manifest (indistinguishable from an ordinary repo, so a hint rather than a diagnosis).
- **Opening the workspace becomes a pipeline step** for workspace commands: build context → open workspace → report load warnings → execute. Commands receive the opened workspace instead of calling `Workspace::find`. Opening failures (not a VMR, bad manifest, old shape) report at this one choke point.
- **Membership.** One module in `src/vmr/`, private to the workspace core but built directly by `init`, owns which Git repositories sit at a VMR root: built from a root and a manifest, it reads the root's first level once (a real directory holding `.git`, named in UTF-8) and holds the cloned children, the missing children (as manifest entries, for clone), and the undeclared repos. A pure constructor takes the listing, so routing tests stay literal paths. `Workspace::open` builds it, `init` builds it over an empty manifest to backfill from its undeclared repos, and the Router asks it who owns a first path component instead of probing the disk. It does not know what is ignored. Children only until 2b.
- **Load warnings** from opening:
  - Declared children missing from disk: one warning naming them and `git vmr clone`. Fan-out ignores them; a path routed into one is an error ("`x` is not cloned").
  - Declared children not ignored: a text scan of `.gitignore` for the exact `/path/` line; only children that miss it go through one `git check-ignore`, so hand-written patterns like `/*/` still count. Warns; never edits.
- **Init** runs `git init` if no repo exists, writes the manifest, backfills it (skipping, with a reason, any path the root repo tracks anything under — the overlap check step 4 reuses) from first-level Git repositories using each one's `origin` (no origin → skipped with a reason; name = path = directory name), and writes their `.gitignore` lines. An existing manifest means "Reinitialized" with no changes — `add` is the way to declare later. Converting an old-style VMR removes the `.gitvmr/` directory before writing the manifest. Init ends with the next step — commit `.gitvmr` and `.gitignore`, then add a remote — since until then the root is unborn and cross-VMR `switch` fails on it. No `--commit` flag.
- **VMR config** (`src/config.rs`, `.gitvmr/config`) is deleted.
- **`.gitignore` comment.** The code that writes ignore lines carries a comment recording why not `.git/info/exclude`: it is shared by every worktree of the root repo while manifests are per branch, so one worktree's view would un-ignore another's children.
- **Fixtures.** `init_vmr(path)` becomes `init_vmr(path, &["backend", "frontend"])`: `git init` on the root plus a manifest with `file://` URLs; children are still created separately, so a missing-child test is a declared name without a matching `init_repo`. Membership is always stated, never inferred. The unit `vmr_fixture()`, the worktree unit tests' hand-built markers, and the benchmark fixture change likewise.

### Step 2a — Member repo type, root excluded

A pure refactor with no expected-output changes.

- `Repo` becomes one struct for every member: `path`, a `label` (`.` or the manifest path) that replaces `name` as the key in outcomes and aggregation, and a `kind` — `Root` or `Child { name }`. A hand-written `Ord` puts the root first, then sorts by path (`.` does not reliably sort first: `-` and `!` precede it). One struct rather than separate types, because nearly all code treats members uniformly; only routing, foreach, and worktrees branch on kind.
- **Router classification.** Each target classifies as:
  - a member repo plus repo-relative path — inside a present child, otherwise the root repo;
  - a child root;
  - inside an undeclared repo — a first-level `.git` not in the manifest (ignored or not), with the path relative to it;
  - an error: outside the VMR, or inside a missing child.

  Generic routing turns a child root into "that child, path `.`" and rejects undeclared repos with today's "not owned by a child Git repository" error until step 4, where it becomes "not declared; `git vmr add` it" as `add` starts declaring, so the other commands' interface is unchanged. `add`, `rm`, and `mv` consume the raw classification.

### Step 2b — Root repo as a member

- `fetch`, `pull`, and `push` skip any member repo with no remote configured, with the reason "no remote", instead of failing — every upgraded VMR starts with a remote-less root repo.
- The root repo joins the member set: fan-out, aggregation scope, routing of root-level paths and `.gitvmr`, and the aggregate path.
- A root-owned path inside a Git repository nested below the first level is an error (a few `stat`s per routed path, done by membership, so the root repo never silently embeds one through an explicit path). Inside a child repo, nesting is that child's business.
- foreach's member environment gives the root `name` and `sm_path` of `.`.
- One mechanical PR: nearly every changed expectation across the ~290 integration tests is "the root repo now appears".

### Step 3 — Clone

- `git vmr clone <url> [<dir>]` clones the root interactively, as today, keeping progress output; reads its manifest; then clones every child in parallel with captured output, each on its remote's default branch, producing repo outcomes.
- Relative URLs (`./`, `../`) resolve against the root's `remote.origin.url` the way `git submodule` does, including scp-style `git@host:path`.
- A failed child clone exits non-zero but leaves the partial VMR in place; `git vmr clone` with no URL fills the gaps through the same child-cloning path, so re-running converges.
- A cloned root without a manifest keeps the clone and errors clearly.
- No pass-through flags yet.

### Step 4 — Declaring and undeclaring

`add` and `rm --cached` run in two stages: membership changes first, then fan-out over the resulting workspace.

- **Stage 1** computes and validates every declaration or undeclaration before writing anything, writes the manifest entries and `.gitignore` lines, and moves the workspace to the new membership in memory — a consuming `declare(changes) -> Workspace` — rather than re-reading the disk. It also adds `.gitvmr` and `.gitignore` to the root repo's pathspecs.
- **Stage 2** is the ordinary fan-out over the resulting membership; the root stages the manifest and ignore edits in its single `git add`, alongside anything else routed to it.
- **What `add` declares.** Top-level repos only; deeper ones are left to git's own behavior (embed plus git's warning) until nesting is supported.
  - Explicit: `add backend` declares `backend` and stages all of its changes; `add backend/file` declares it and stages that file. Name = path; URL from `origin`, and an explicit operand without one is an error with no writes.
  - Implicit: a scope covering undeclared, unignored repos (`add -A`) declares them all and stages each one's changes. One without an `origin` is excluded from staging (`:(exclude)/x/`) with a warning to set one or ignore it.
  - Ignored repos are never declared implicitly — ignoring is how a repo stays local. Explicitly adding into one fails ("`scratch` is ignored and not declared") unless `-f` is given, which declares it.
- **What `rm --cached` undeclares.** Only an exact child root: it removes the `[repo.<name>]` table and the exact `/path/` line, and stage 2 runs over the reduced membership. `rm --cached backend/file` stays an ordinary in-child removal; aggregate `rm --cached -r .` undeclares nothing. Output notes that the directory remains and will be declared again by `add -A` unless deleted or ignored — the same as git's `rm --cached` then `add -A`.
- **Overlap.** Declaring refuses, using the check built for `init` in step 1, a path the root repo tracks anything under (`git ls-files -- <path>`), since ignore rules do not untrack files and both repos would claim the directory. Hand-edited manifests are not checked: workspace open runs no overlap check.
- **Refusals.** Plain `rm` on a child root is refused with a hint. `mv` refuses a child root as source or destination ("moving child repos is not supported yet; edit the manifest path and move the directory") — a real rename must also repair the moved repo's linked worktrees.
- **Reporting.** Each declaration is a success message on the root repo ("declared `x`"); `-n` reports "would declare `x`" / "would undeclare `x`", writes nothing in stage 1, and dry-runs stage 2 in the to-be-declared repos, which already exist on disk.
- **Partial failure.** If stage 2 fails somewhere, the declarations stand and the failure reports as usual; re-running converges. No rollback.
- Staging `.gitvmr` and `.gitignore` whole also stages unrelated unstaged edits in them, as `git submodule add` does with `.gitmodules`.

### Step 5 — Worktrees

- **Add.** `git worktree add` in the root repo first (branch from `-b`, a commit-ish, or the target basename, as today); if it fails, nothing else is attempted. Then read the new worktree's manifest and, for each child it declares, match the main VMR's child by name and run `git worktree add <target>/<path>` there. Matching by name finds a child even if its path differs on that branch. Children declared there but not cloned here are skipped with a reason.
- **Remove.** Child worktrees first; the root's `git worktree remove` runs only if every child succeeded, otherwise the root is deliberately kept. This gate is essential — verified: `git worktree remove` on the root succeeds while a child worktree is still inside it, deletes it as ignored content, and leaves the child repo with a prunable worktree record. `--delete` covers the root repo's branch too.
- **Move.** One `git worktree move` of the root worktree — carrying the child worktree directories with it — then `git worktree repair <new>/<path>` in each child repo. Retrying after a partial failure converges, since the repairs are idempotent. Verified against git 2.55: `git worktree move` carries nested child worktrees along, and `repair` fixes each one (reporting `gitdir incorrect` on stderr as it does) — treat that output as success.
- **List.** Roots come from the root repo's `git worktree list`; child worktrees group under them by `<root>/<path>`, replacing the `owning_root` heuristic.

## Impact

- `src/vmr.rs`, `src/vmr/repo.rs`: discovery, marker lifecycle, and repo scanning become manifest reading. `src/config.rs`: VMR config removed. `src/store.rs` untouched.
- `src/cli/pipeline.rs`, `src/cli/context.rs`, `src/commands.rs`: the workspace opens as a pipeline step and contributes load warnings.
- `src/workspace.rs`, `src/workspace/routing.rs`: the member set, the declare transition, and Router classification.
- `src/git.rs`, `src/render/outcomes.rs`: outcomes and aggregation keyed by label.
- `src/commands/{init,clone,add,rm,mv,foreach,worktree}.rs` and `src/commands/worktree/roots.rs`.
- `Cargo.toml`: new `toml_edit` dependency.
- `tests/common/mod.rs`, `src/test_support.rs`, `benches/real_repos.rs`: fixtures build a root repo with a manifest.
- `GLOSSARY.md` (VMR, root repo, member repo, manifest, declare/undeclare, workspace, member environment, worktree root, materialize/dissolve) and ADR 0004 record the domain decisions.
