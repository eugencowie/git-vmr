## 1. CLI Parsing

- [x] 1.1 Add a `-d`/`--delete` flag to the `Tag` CLI variant that requires `tag_name`.
- [x] 1.2 Dispatch `git vmr tag -d <tagname>` and `git vmr tag --delete <tagname>` to a tag deletion command path.
- [x] 1.3 Preserve existing dispatch for `git vmr tag` list mode and `git vmr tag <tagname>` lightweight creation mode.
- [x] 1.4 Add CLI parser tests for short delete, long delete, missing tag name, multi-tag deletion rejection, and existing unsupported tag options.

## 2. Git Execution

- [x] 2.1 Add a Git tag deletion helper that runs `git tag -d <tagname>` in a child repository.
- [x] 2.2 Preserve Git's first successful deletion output line and suffix it with the repository name.
- [x] 2.3 Preserve Git's first failure output line and suffix it with the repository name.
- [x] 2.4 Export the tag deletion helper through the existing Git module boundary.

## 3. Aggregate Tag Command

- [x] 3.1 Add a tag deletion function that discovers the VMR root and immediate child Git repositories.
- [x] 3.2 Attempt tag deletion independently in every discovered child Git repository.
- [x] 3.3 Execute deletion attempts in parallel while capturing each repository result.
- [x] 3.4 Return non-zero after all deletion attempts finish when any repository fails.
- [x] 3.5 Keep tag listing and lightweight tag creation behavior unchanged.

## 4. Tests

- [x] 4.1 Add an integration test that `git vmr tag -d <tagname>` deletes the tag in every child Git repository.
- [x] 4.2 Add an integration test that `git vmr tag --delete <tagname>` deletes the tag in every child Git repository.
- [x] 4.3 Add an integration test showing non-Git child directories are skipped during tag deletion.
- [x] 4.4 Add an integration test showing partial deletion failures do not stop successful deletion in other repositories.
- [x] 4.5 Add an integration test for concise missing-tag failure reports with repository suffixes.
- [x] 4.6 Add an integration test proving multiple deletion failures are reported in deterministic repository-name order.
- [x] 4.7 Run existing tag listing and tag creation tests to verify unchanged behavior.

## 5. Documentation

- [x] 5.1 Update `docs/git-vmr/tag.md` to mark `-d`/`--delete` as supported for single-tag deletion.
- [x] 5.2 Document that multi-tag deletion, force replacement, annotated/signed tags, explicit target objects, verification, filtering, formatting, and patterns remain unsupported.

## 6. Validation

- [x] 6.1 Run `nix develop -c cargo fmt --check`.
- [x] 6.2 Run `nix develop -c cargo test`.
- [x] 6.3 Run `nix develop -c openspec validate tag-delete-argument --strict`.
