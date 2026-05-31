## 1. Release Preparation

- [ ] 1.1 Add release-plz configuration for the `develop` release branch,
  changelog maintenance, crates.io publication, and release-PR-gated publishing.
- [ ] 1.2 Add a committed `CHANGELOG.md` suitable for release-plz maintenance.
- [ ] 1.3 Add the release-plz GitHub Actions workflow for release pull request
  preparation and approved release publication.

## 2. Binary Distribution

- [ ] 2.1 Initialize dist configuration for `aarch64-apple-darwin`,
  `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`, and
  `x86_64-pc-windows-msvc`, excluding Linux musl targets.
- [ ] 2.2 Commit the generated dist release workflow and verify that its trigger
  responds to release-plz-generated version tags.
- [ ] 2.3 Run the dist configuration validation or regeneration command locally
  and verify that committed generated files are current.

## 3. Project Site

- [ ] 3.1 Add pinned Oranda configuration for the project homepage and dist
  installer presentation.
- [ ] 3.2 Add a GitHub Pages workflow that builds and deploys the Oranda site
  independently for relevant `develop` changes and completed releases.
- [ ] 3.3 Build the Oranda site locally and verify that the generated site
  includes release and installation information.

## 4. Maintenance Workflow

- [ ] 4.1 Add `mise` tasks or documented commands for release configuration
  validation, dist workflow regeneration, and local Oranda builds.
- [ ] 4.2 Document the supported artifact matrix, release-PR flow, generated-file
  maintenance, and rollback procedure.
- [ ] 4.3 Validate GitHub Actions workflow syntax and run the existing aggregate
  local validation task.

## 5. Repository Bootstrap

- [ ] 5.1 Choose and document the intended initial public crate version before
  enabling automated publication.
- [ ] 5.2 Configure a repository-scoped fine-grained PAT or GitHub App token for
  release-plz so generated pull requests and version tags trigger downstream
  workflows.
- [ ] 5.3 Configure a scoped crates.io token and perform the controlled first
  publication to reserve the crate name without publishing placeholder version
  `0.0.0`.
- [ ] 5.4 Enable the release workflows and verify that release-plz can open a
  release pull request against `develop`.
- [ ] 5.5 Merge a reviewed release pull request and verify crates.io
  publication, GitHub Release creation, dist artifact uploads for all supported
  targets, and GitHub Pages deployment.
- [ ] 5.6 Replace the crates.io token with trusted publishing after the initial
  crate publication when the selected release-plz integration supports it.
