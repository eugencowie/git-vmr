## Why

The project has automated validation but no repeatable process for producing
versioned releases, installable binaries, or a public project site. Establishing
the release pipeline now prevents ad hoc publication steps from becoming part of
the project's operating model.

## What Changes

- Add release pull requests that update the crate version and changelog from
  changes merged into `develop`.
- Publish approved releases to crates.io and GitHub Releases after the release
  pull request is merged.
- Build downloadable archives, checksums, and installers for macOS Intel, macOS
  Apple Silicon, Linux x86_64 GNU, and Windows x86_64.
- Publish a landing page to GitHub Pages with release and installation
  information.
- Document the one-time crates.io bootstrap and the credentials required for
  automated releases.

## Capabilities

### New Capabilities

- `release-process`: Define automated release preparation, publication,
  artifact distribution, project-site deployment, and release bootstrap
  requirements.

### Modified Capabilities

None.

## Impact

- Add release, artifact-distribution, and documentation-site configuration at
  repository root.
- Add GitHub Actions workflows for release-plz, dist, and GitHub Pages.
- Add a committed changelog maintained by release-plz.
- Add release-oriented local tooling tasks where useful for validation.
- Require GitHub repository secrets or equivalent release credentials for
  crates.io publication and downstream workflow triggering.
