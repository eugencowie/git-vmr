## 1. Release Preparation

- [x] 1.1 Add release-plz configuration for the `develop` release branch,
  standalone git-cliff changelog maintenance, crates.io publication, and
  release-PR-gated publishing.
- [x] 1.2 Generate `CHANGELOG.md` with standalone git-cliff as part of the
  release workflow.
- [x] 1.3 Add the release-plz GitHub Actions workflow for release pull request
  preparation and approved release publication.

## 2. Binary Distribution

- [x] 2.1 Initialize dist configuration for `aarch64-apple-darwin`,
  `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`, and
  `x86_64-pc-windows-msvc`, excluding Linux musl targets.
- [x] 2.2 Commit the generated dist release workflow and verify that its trigger
  responds to release-plz-generated version tags.
- [x] 2.3 Run the dist configuration validation or regeneration command locally
  and verify that committed generated files are current.

## 3. Project Site

- [x] 3.1 Add pinned Oranda configuration for the project homepage and dist
  installer presentation.
- [x] 3.2 Add a GitHub Pages workflow that builds and deploys the Oranda site
  independently for relevant `develop` changes and completed releases.
- [x] 3.3 Build the Oranda site locally.

## 4. Maintenance Workflow

- [x] 4.1 Add `mise` tasks or documented commands for release configuration
  validation and local Oranda builds.
- [x] 4.2 Validate GitHub Actions workflow syntax and run the existing aggregate
  local validation task.

## 5. Repository Bootstrap

- [x] 5.1 Configure a repository-scoped fine-grained PAT or GitHub App token for
  release-plz so generated pull requests and version tags trigger downstream
  workflows.
- [x] 5.2 Configure a scoped crates.io token without publishing placeholder
  version `0.0.0`.
