//! Shared unit-test fixtures.

use crate::cli::CliContext;
use crate::git::Git;
use crate::vmr::{
    Declaration, Gitignore, Manifest, ManifestEditor, Repo, RepoKind, RepoPath,
    RepoUrl
};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// A child repo value, labelled and named `label`, for exercising git ops
/// directly.
pub(crate) fn repo(label: &str, path: impl Into<PathBuf>) -> Repo
{
    Repo {
        path: path.into(),
        label: label.to_owned(),
        kind: RepoKind::Child { name: label.to_owned() }
    }
}

/// A CLI context rooted at `working_dir` whose git is never invoked —
/// workspace commands reach git through the workspace, not the context.
pub(crate) fn cli_context(working_dir: &std::path::Path) -> CliContext
{
    CliContext::for_tests(working_dir, Git::subprocess())
}

/// Materializes a minimal VMR on disk: a root repo whose manifest declares
/// two cloned and ignored child repos, `backend` and `frontend`.
pub(crate) fn vmr_fixture() -> TempDir
{
    let tmp = tempfile::tempdir().unwrap();
    root_repo(tmp.path(), &["backend", "frontend"], &["backend", "frontend"]);
    fs::create_dir_all(tmp.path().join("backend/.git")).unwrap();
    fs::create_dir_all(tmp.path().join("frontend/.git")).unwrap();
    tmp
}

/// Makes `dir` a root repo, as far as discovery can tell: a `.git`
/// directory, a manifest declaring `declared` (name = path), and a
/// `.gitignore` with an anchored line for each of `ignored`.
pub(crate) fn root_repo(dir: &Path, declared: &[&str], ignored: &[&str])
{
    // Mark as a Git repository
    fs::create_dir_all(dir.join(".git")).unwrap();

    // Write manifest
    fs::write(dir.join(".gitvmr"), manifest_text(declared)).unwrap();

    // Ignore child repos
    Gitignore::read(dir).ignore(ignored.iter().copied()).unwrap();
}

/// A manifest declaring `declared` (name = path).
pub(crate) fn manifest(declared: &[&str]) -> Manifest
{
    Manifest::parse(&manifest_text(declared)).unwrap()
}

fn manifest_text(declared: &[&str]) -> String
{
    let mut editor = ManifestEditor::new();
    for path in declared
    {
        editor.declare(&Declaration {
            name: (*path).to_owned(),
            path: RepoPath::parse((*path).to_owned()).unwrap(),
            url: RepoUrl::parse(format!("../{path}.git")).unwrap()
        });
    }
    editor.to_string()
}
