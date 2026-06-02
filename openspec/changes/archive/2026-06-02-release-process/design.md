## Context

The repository contains a single Rust binary crate and validates changes through
GitHub Actions, but it does not yet prepare releases, publish packages, attach
binary artifacts, or deploy the homepage declared in Cargo metadata. Releases
will be cut from `develop`.

Three tools divide the release concerns:

- release-plz prepares version pull requests and publishes approved crate
  releases.
- git-cliff generates the changelog from merged conventional commits and adds it
  to release pull requests.
- dist, formerly cargo-dist, builds binary artifacts and installers and attaches
  them to GitHub Releases.
- Oranda builds the project site and presents release installation options.

Automated crates.io publication requires release credentials. In addition, a
tag created with GitHub's default workflow token does not trigger a second
tag-based workflow, so the release-plz workflow needs a credential that permits
the generated tag to dispatch the dist workflow.

## Goals / Non-Goals

**Goals:**

- Prepare releases through reviewable pull requests against `develop`.
- Publish approved versions to crates.io and GitHub Releases.
- Produce binary archives, checksums, and installers for the supported platform
  matrix.
- Deploy a public Oranda site to GitHub Pages.
- Keep generated workflow files reproducible from committed configuration.
- Document bootstrap actions and required repository credentials.

**Non-Goals:**

- Publish Linux musl artifacts.
- Publish Linux ARM artifacts.
- Add package-manager distribution such as Homebrew, Scoop, or winget.
- Automate signing, notarization, or provenance attestations in the first
  iteration.
- Change the existing validation workflow beyond adjustments required to make
  release pull requests and generated workflows operate correctly.

## Decisions

### Use release-plz for release preparation and publication

Add release-plz configuration and a GitHub Actions workflow with separate
release pull request and release jobs. Disable release-plz's package-scoped
changelog generation. Before release-plz opens or updates a release pull
request, generate `CHANGELOG.md` from the merged `develop` history with
standalone git-cliff. Configure release-plz to include that dirty generated file
in its pull request update. The release job runs after changes land on `develop`
and publishes only when release-plz identifies an approved release.

Configure `release_always = false` so ordinary merges do not publish a release.
Publishing remains gated by merging the generated release pull request.

Alternative considered: publish directly from manually pushed tags. That would
reduce configuration but would make changelog and version updates manual and
remove the reviewable release gate.

### Publish the crate to crates.io and bootstrap the first publication

Keep crates.io publication enabled. The initial crates.io publication requires
a scoped crates.io token because trusted publishing cannot create a new crate.
After the crate exists on crates.io, migrate to crates.io trusted publishing if
the selected release-plz integration supports the repository workflow.

Alternative considered: GitHub-only binary releases with `git_only = true`.
This was rejected because crates.io publication is part of the desired release
surface.

### Use a release credential that triggers downstream workflows

Provide the release-plz workflow with a fine-grained personal access token or
GitHub App token that can create release pull requests and tags and can trigger
the dist workflow from the generated tag. The dist workflow creates the GitHub
Release after building its artifacts. Keep the release-plz credential narrowly
scoped to this repository.

Alternative considered: use only GitHub's default workflow token and invoke
dist from inside the release-plz workflow. This would couple the workflows and
depart from dist's generated release workflow model.

### Generate binary artifacts with dist

Initialize dist and commit its generated release workflow. Configure the
following target matrix:

| Platform | Target |
| --- | --- |
| macOS Apple Silicon | `aarch64-apple-darwin` |
| macOS Intel | `x86_64-apple-darwin` |
| Linux x86_64 GNU | `x86_64-unknown-linux-gnu` |
| Windows x86_64 | `x86_64-pc-windows-msvc` |

Generate archives, checksums, and shell and PowerShell installers where
supported. Do not generate Linux musl artifacts.

Alternative considered: maintain a handwritten artifact workflow. This was
rejected because dist already owns the platform build graph and installer
generation.

### Deploy Oranda independently from release publication

Add Oranda configuration and a GitHub Pages workflow that builds the site and
uploads the Pages artifact. Trigger deployment for documentation changes on
`develop` and after release completion so the website is refreshed without
blocking binary publication.

Pin the Oranda version used in automation because the site generator is not on
the critical release path and should change only through explicit maintenance.

Alternative considered: treat site publication as a required stage of the
release workflow. This was rejected because a documentation-site failure should
not prevent release artifacts from being published.

### Keep committed configuration as the source of truth

Commit release-plz configuration, dist configuration, Oranda configuration,
`CHANGELOG.md`, generated workflows, and documentation for repository
credentials and bootstrap steps. Add local `mise` tasks where they provide
useful validation or regeneration commands.

## Risks / Trade-offs

- [Release credentials are broader than the default workflow token] -> Use a
  repository-scoped fine-grained PAT or GitHub App token with the minimum
  permissions required and document rotation.
- [The initial automated run could publish the placeholder `0.0.0` version] ->
  Complete and verify the crates.io bootstrap and first intended release version
  before enabling publication automation.
- [Generated dist workflow output can drift from committed configuration] ->
  Document the regeneration command and validate that regenerated output is
  clean during implementation.
- [Changelog generation dirties the release-plz checkout] -> Enable
  release-plz's `allow_dirty` option so the generated changelog is included in
  the release pull request update.
- [Oranda version age can introduce maintenance constraints] -> Pin the version
  and keep Pages deployment independent from package publication.
- [Unsupported platforms require manual installation from source] -> Document
  the initial supported matrix and extend it only when demand justifies the
  additional CI cost.

## Migration Plan

1. Add committed release-plz, dist, Oranda, changelog, and workflow
   configuration without enabling an accidental publication.
2. Create repository credentials for GitHub release automation and the first
   crates.io publication.
3. Reserve the crate name on crates.io by performing the controlled first
   publication at the intended initial version.
4. Enable release automation and verify that release-plz can open a release pull
   request against `develop`.
5. Merge a reviewed release pull request and verify crates.io publication,
   GitHub Release creation, dist artifact uploads, and Oranda Pages deployment.
6. Replace the crates.io token with trusted publishing when supported and
   verified.

Rollback consists of disabling the release and Pages workflows. Published crate
versions and GitHub Release tags remain immutable release records.

## Open Questions

- Which intended version should be used for the first controlled crates.io
  publication?
- Should the GitHub release automation credential be a fine-grained personal
  access token or a GitHub App token?
