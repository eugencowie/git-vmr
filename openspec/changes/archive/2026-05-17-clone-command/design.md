## Context

`git-vmr` is installed as `git-vmr` so users can run it as `git vmr <subcommand>`. Existing implemented commands either create VMR metadata (`init`) or operate across immediate child Git repositories discovered from a `.gitvmr/` root. `clone` is different: it creates a new Git work area before any VMR root exists, and Git already owns the full clone user experience, including transport, authentication, progress, and destination-directory behavior.

## Goals / Non-Goals

**Goals:**

- Add `git vmr clone <repo> [<dir>]` as a narrow passthrough to `git clone`.
- Honor the existing global `-C <path>` option by invoking Git from the effective working directory.
- Preserve Git's clone stdout, stderr, progress output, prompts, and exit status.
- Avoid requiring `.gitvmr/` discovery before cloning.
- Keep the initial argument surface intentionally small.

**Non-Goals:**

- Do not initialize `.gitvmr/` in the cloned repository.
- Do not clone multiple child repositories or interpret VMR configuration.
- Do not rewrite repository URLs, destination paths, branches, remotes, or clone options.
- Do not add passthrough support for arbitrary `git clone` flags in the first version.

## Decisions

### Decision: Model clone as a passthrough command

The command should invoke `git -C <working-dir> clone <repo> [<dir>]` and return Git's process status. This matches user expectations for clone and keeps repository transport, authentication, progress, and destination-name inference inside Git.

Alternative considered: implement clone with the existing `git_output` helper and summarize its output. That helper captures stdout and stderr, which is useful for aggregate VMR commands but worse for clone because users often need live progress and interactive credential prompts.

### Decision: Keep clone outside VMR root discovery

The command should not call VMR root discovery. `clone` is a startup command and must work in ordinary directories, including directories that are not Git repositories and do not contain `.gitvmr/`.

Alternative considered: require a VMR root and clone into it as a child repository. That may be useful later, but it is not a simple passthrough and would create questions about VMR metadata, child naming, and configured repository membership.

### Decision: Support only `<repo> [<dir>]` initially

The initial parser should accept a repository argument and an optional destination directory argument. Additional `git clone` flags can be added later as explicit supported behavior or as a broader trailing-argument passthrough.

Alternative considered: collect all trailing arguments after `clone` and forward them to Git. That would support more Git syntax immediately, but it weakens the CLI contract and complicates clap parsing, validation, help text, and tests.

### Decision: Reuse global `-C` semantics

The existing CLI already canonicalizes `-C <path>` before dispatch. `clone` should use that effective directory as Git's working directory so relative destination directories behave the same as `git -C <path> clone <repo> [<dir>]`.

Alternative considered: pass `-C` through only when the user provided it and otherwise inherit the process current directory. That creates two execution paths for no user-visible benefit.

## Risks / Trade-offs

- Limited clone flag support may surprise users who expect complete `git clone` parity -> Document the supported syntax and keep broader passthrough as a future extension.
- Inherited stdio makes output less convenient to assert in tests -> Prefer behavior fidelity for clone and test filesystem effects plus exit status.
- `git -C <working-dir> clone` depends on Git's own path handling -> This is intentional passthrough behavior and should be covered with `-C` destination tests.
