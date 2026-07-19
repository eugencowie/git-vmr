mod common;

use common::{git, git_vmr, init_repo, init_vmr, write_commit};
use predicates::prelude::*;
use std::fs;
use std::path::Path;

#[test]
fn diff_shows_worktree_changes_across_child_repos()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    init_repo(&backend);
    init_repo(&frontend);
    write_commit(&backend, "src.rs", "old\n", "initial");
    write_commit(&frontend, "app.js", "old\n", "initial");
    fs::write(backend.join("src.rs"), "new\n").expect("failed to write file");
    fs::write(frontend.join("app.js"), "new\n").expect("failed to write file");

    // A poisoned pager proves piped output never consults one; piped
    // stdout also means uncoloured output by contract
    git_vmr()
        .current_dir(tmp.path())
        .env("GIT_PAGER", "false")
        .arg("diff")
        .assert()
        .success()
        .stdout(
            predicate::str::contains(
                "diff --git a/backend/src.rs b/backend/src.rs"
            )
            .and(predicate::str::contains(
                "diff --git a/frontend/app.js b/frontend/app.js"
            ))
            .and(predicate::str::contains("\u{1b}").not())
        )
        .stderr(predicate::str::is_empty());
}

#[test]
fn diff_staged_shows_index_changes_only()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    init_repo(&backend);
    init_repo(&frontend);
    write_commit(&backend, "src.rs", "old\n", "initial");
    write_commit(&frontend, "app.js", "old\n", "initial");
    fs::write(backend.join("src.rs"), "staged\n")
        .expect("failed to write file");
    git(&backend, ["add", "src.rs"]);
    fs::write(frontend.join("app.js"), "unstaged\n")
        .expect("failed to write file");

    git_vmr()
        .current_dir(tmp.path())
        .args(["diff", "--staged"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("a/backend/src.rs")
                .and(predicate::str::contains("frontend").not())
        )
        .stderr(predicate::str::is_empty());
}

#[test]
fn diff_routes_paths_to_the_owning_repo()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    init_repo(&backend);
    init_repo(&frontend);
    write_commit(&backend, "src.rs", "old\n", "initial");
    write_commit(&frontend, "app.js", "old\n", "initial");
    fs::write(backend.join("src.rs"), "new\n").expect("failed to write file");
    fs::write(frontend.join("app.js"), "new\n").expect("failed to write file");

    git_vmr()
        .current_dir(tmp.path())
        .args(["diff", "backend/src.rs"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("a/backend/src.rs")
                .and(predicate::str::contains("frontend").not())
        )
        .stderr(predicate::str::is_empty());
}

#[test]
fn diff_aggregate_path_expands_to_all_child_repos()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    init_repo(&backend);
    init_repo(&frontend);
    write_commit(&backend, "src.rs", "old\n", "initial");
    write_commit(&frontend, "app.js", "old\n", "initial");
    fs::write(backend.join("src.rs"), "new\n").expect("failed to write file");
    fs::write(frontend.join("app.js"), "new\n").expect("failed to write file");

    git_vmr()
        .current_dir(tmp.path())
        .args(["diff", "."])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("a/backend/src.rs")
                .and(predicate::str::contains("a/frontend/app.js"))
        )
        .stderr(predicate::str::is_empty());
}

#[test]
fn diff_errors_on_a_path_owned_by_no_child_repo()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let backend = tmp.path().join("backend");
    init_repo(&backend);
    write_commit(&backend, "src.rs", "old\n", "initial");
    fs::create_dir(tmp.path().join("docs")).expect("failed to create dir");
    fs::write(tmp.path().join("docs/readme.md"), "text\n")
        .expect("failed to write file");

    git_vmr()
        .current_dir(tmp.path())
        .args(["diff", "docs/readme.md"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("child Git repository"));
}

#[test]
fn diff_aggregates_a_failing_repo_while_surviving_diffs_still_print()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let broken = tmp.path().join("broken");
    let frontend = tmp.path().join("frontend");
    fs::create_dir(&broken).expect("failed to create repo dir");
    fs::write(broken.join(".git"), "gitdir: /nonexistent/gitdir\n")
        .expect("failed to corrupt git dir");
    init_repo(&frontend);
    write_commit(&frontend, "app.js", "old\n", "initial");
    fs::write(frontend.join("app.js"), "new\n").expect("failed to write file");

    git_vmr()
        .current_dir(tmp.path())
        .arg("diff")
        .assert()
        .failure()
        .stdout(predicate::str::contains("a/frontend/app.js"))
        .stderr(
            predicate::str::contains("diff failed")
                .and(predicate::str::contains("broken"))
        );
}

#[test]
fn diff_with_no_pager_prints_directly()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let backend = tmp.path().join("backend");
    init_repo(&backend);
    write_commit(&backend, "src.rs", "old\n", "initial");
    fs::write(backend.join("src.rs"), "new\n").expect("failed to write file");

    git_vmr()
        .current_dir(tmp.path())
        .env("GIT_PAGER", "false")
        .args(["diff", "--no-pager"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("a/backend/src.rs")
                .and(predicate::str::contains("\u{1b}").not())
        )
        .stderr(predicate::str::is_empty());
}

// The applyability contract from the spec: when stdout is piped, the
// combined output is a patch `git apply -p1` accepts from the VMR root.
// Real edits — a text change, a pure cross-directory rename, a quoted-path
// rename, a binary change, and a mode-only change — go through
// the real binary, the captured stdout applies onto a pristine copy of the
// VMR, and the trees must match. One assertion, no golden file.
#[test]
fn diff_output_round_trips_through_git_apply()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = tmp.path().join("vmr");
    fs::create_dir(&vmr).expect("failed to create vmr dir");
    init_vmr(&vmr);

    // The non-ASCII character forces Git C-quoting in prefixed paths while
    // remaining a valid Windows filename; the quoted-path rename also makes
    // the prefix rewrite insert the repository into quoted headers.
    let backend = vmr.join("backénd");
    let frontend = vmr.join("frontend");
    init_repo(&backend);
    init_repo(&frontend);
    git(&backend, ["config", "core.quotePath", "true"]);
    write_commit(&backend, "src/kept.rs", "line one\nline two\n", "initial");
    write_commit(
        &backend,
        ".gitattributes",
        "src/kept.rs diff=roundtrip\n",
        "configure textconv attribute"
    );
    git(&backend, [
        "config",
        "diff.roundtrip.textconv",
        "sed s/line/textconv/g"
    ]);
    write_commit(&backend, "src/old.rs", "moved verbatim\n", "add old");
    write_commit(&backend, "quoteéfile.txt", "quoted path\n", "add quoted");
    write_commit(&backend, "script.sh", "#!/bin/sh\n", "add script");
    fs::write(backend.join("logo.bin"), [0u8, 159, 146, 150, 10])
        .expect("failed to write binary file");
    git(&backend, ["add", "logo.bin"]);
    git(&backend, ["commit", "-m", "add binary"]);
    write_commit(&frontend, "app.js", "console.log('old')\n", "initial");

    let pristine = tmp.path().join("pristine");
    copy_tree(&vmr, &pristine);

    // Real edits, staged so rename detection sees each delete/add pair
    fs::write(backend.join("src/kept.rs"), "line one\nline changed\n")
        .expect("failed to write file");
    fs::create_dir(backend.join("lib")).expect("failed to create dir");
    git(&backend, ["mv", "src/old.rs", "lib/new.rs"]);
    git(&backend, ["mv", "quoteéfile.txt", "newquoteéfile.txt"]);
    fs::write(backend.join("logo.bin"), [255u8, 216, 255, 224, 0])
        .expect("failed to write binary file");
    make_executable(&backend.join("script.sh"));
    git(&backend, ["add", "-A"]);
    fs::write(frontend.join("app.js"), "console.log('new')\n")
        .expect("failed to write file");
    git(&frontend, ["add", "-A"]);

    let output = git_vmr()
        .current_dir(&vmr)
        .args(["diff", "--staged"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert!(!output.is_empty(), "expected a non-empty combined diff");

    let patch = tmp.path().join("combined.patch");
    fs::write(&patch, &output).expect("failed to write patch");
    git(&pristine, [
        "-c",
        "core.autocrlf=false",
        "apply",
        "-p1",
        patch.to_str().expect("patch path should be UTF-8")
    ]);

    assert_trees_match(&vmr, &pristine);
}

/// Copies a directory tree, preserving file contents and permissions.
fn copy_tree(source: &Path, destination: &Path)
{
    fs::create_dir(destination).expect("failed to create directory");
    for entry in fs::read_dir(source).expect("failed to read directory")
    {
        let entry = entry.expect("failed to read directory entry");
        let target = destination.join(entry.file_name());
        if entry.file_type().expect("failed to stat entry").is_dir()
        {
            copy_tree(&entry.path(), &target);
        }
        else
        {
            fs::copy(entry.path(), &target).expect("failed to copy file");
        }
    }
}

/// Asserts two worktrees hold the same files with the same bytes and the
/// same executable bit, ignoring `.git` state.
fn assert_trees_match(a: &Path, b: &Path)
{
    let mut a_files = list_files(a);
    let mut b_files = list_files(b);
    a_files.sort();
    b_files.sort();
    assert_eq!(a_files, b_files, "trees differ in file sets");

    for file in a_files
    {
        let a_path = a.join(&file);
        let b_path = b.join(&file);
        assert_eq!(
            fs::read(&a_path).expect("failed to read file"),
            fs::read(&b_path).expect("failed to read file"),
            "contents differ for '{file}'"
        );
        assert_eq!(
            is_executable(&a_path),
            is_executable(&b_path),
            "executable bit differs for '{file}'"
        );
    }
}

/// Worktree-relative paths of every file under `root`, skipping `.git`.
fn list_files(root: &Path) -> Vec<String>
{
    fn walk(root: &Path, dir: &Path, files: &mut Vec<String>)
    {
        for entry in fs::read_dir(dir).expect("failed to read directory")
        {
            let entry = entry.expect("failed to read directory entry");
            if entry.file_name() == ".git"
            {
                continue;
            }
            let path = entry.path();
            if entry.file_type().expect("failed to stat entry").is_dir()
            {
                walk(root, &path, files);
            }
            else
            {
                files.push(
                    path.strip_prefix(root)
                        .expect("path is under root")
                        .to_str()
                        .expect("path should be UTF-8")
                        .to_owned()
                );
            }
        }
    }

    let mut files = Vec::new();
    walk(root, root, &mut files);
    files
}

#[cfg(unix)]
fn make_executable(path: &Path)
{
    use std::os::unix::fs::PermissionsExt;
    let mut permissions =
        fs::metadata(path).expect("failed to stat file").permissions();
    permissions.set_mode(permissions.mode() | 0o111);
    fs::set_permissions(path, permissions).expect("failed to chmod file");
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool
{
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(path).expect("failed to stat file").permissions().mode()
        & 0o100
        != 0
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) {}

#[cfg(not(unix))]
fn is_executable(_path: &Path) -> bool
{
    false
}
