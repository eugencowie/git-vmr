## Purpose

Define the automated release preparation, package publication, binary
distribution, project site deployment, and controlled bootstrap process for
`git-vmr`.

## Requirements

### Requirement: Reviewable release preparation
The repository SHALL use release-plz to prepare release pull requests against
`develop` that update the crate version, SHALL use standalone git-cliff to add a
changelog generated from merged conventional commits to those pull requests,
and SHALL publish a release only after the corresponding release pull request
has been merged.

#### Scenario: Prepare a release pull request
- **WHEN** releasable changes have been merged into `develop`
- **THEN** automation opens or updates a release pull request containing the
  proposed crate version and a git-cliff changelog generated from merged
  conventional commits

#### Scenario: Keep an ordinary merge unpublished
- **WHEN** a non-release pull request is merged into `develop`
- **THEN** automation does not publish a new crate version unless an approved
  release pull request has established the release

### Requirement: Crates.io and GitHub publication
The repository SHALL publish an approved release to crates.io and create a
corresponding tagged GitHub Release.

#### Scenario: Publish an approved release
- **WHEN** an approved release pull request is merged into `develop`
- **THEN** automation publishes the crate version to crates.io and creates the
  corresponding version tag and GitHub Release

### Requirement: Distributed binary artifacts
The repository SHALL use dist to attach binary archives, checksums, and
supported installers to each GitHub Release for `aarch64-apple-darwin`,
`x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`, and
`x86_64-pc-windows-msvc`.

#### Scenario: Build release artifacts
- **WHEN** release-plz creates a version tag for an approved release
- **THEN** the dist workflow builds and uploads release artifacts for each
  supported target

#### Scenario: Exclude Linux musl artifacts
- **WHEN** the dist workflow builds release artifacts
- **THEN** it does not build or upload an artifact for a Linux musl target

### Requirement: Release workflow dispatch
The repository SHALL document and configure a repository-scoped GitHub release
credential that allows a release tag created by release-plz automation to
trigger the downstream dist workflow.

#### Scenario: Trigger artifact generation from an automated release
- **WHEN** release-plz automation creates the version tag using the configured
  release credential
- **THEN** GitHub dispatches the dist workflow for that tag

### Requirement: Public project site
The repository SHALL use Oranda to build a project site with release and
installation information and SHALL deploy the generated site to GitHub Pages
independently of package publication.

#### Scenario: Deploy documentation changes
- **WHEN** site-relevant changes are merged into `develop`
- **THEN** automation builds the Oranda site and deploys it to GitHub Pages

#### Scenario: Refresh the site after release completion
- **WHEN** a GitHub Release completes
- **THEN** automation rebuilds and deploys the Oranda site with current release
  information

#### Scenario: Keep release publication independent
- **WHEN** Oranda site deployment fails
- **THEN** crates.io publication and GitHub Release artifact generation are not
  blocked or rolled back

### Requirement: Controlled release bootstrap
The repository SHALL document a controlled bootstrap procedure for the initial
crates.io publication and SHALL prevent release automation from publishing the
placeholder `0.0.0` version unintentionally.

#### Scenario: Bootstrap the initial crates.io version
- **WHEN** release automation is enabled for the first time
- **THEN** the crate name has been reserved through a controlled first
  publication at the intended initial version and the placeholder `0.0.0`
  version is not published accidentally

### Requirement: Reproducible release configuration
The repository SHALL commit release-plz configuration, dist configuration,
Oranda configuration, generated release workflows, and the changelog, and
SHALL document commands needed to validate or regenerate generated release
files.

#### Scenario: Inspect release configuration
- **WHEN** a developer reviews the checked-out repository
- **THEN** the release configuration, changelog, workflows, and regeneration
  instructions required to maintain the release process are available without
  inspecting another branch
