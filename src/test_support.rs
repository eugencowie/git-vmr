//! Shared unit-test fixtures.

use crate::cli::CliContext;
use crate::git::Git;
use crate::vmr::Repo;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// A child repo value for exercising git ops directly.
pub(crate) fn repo(name: &str, path: &str) -> Repo
{
    Repo { name: name.to_owned(), path: PathBuf::from(path) }
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
    fs::create_dir_all(dir.join(".git")).unwrap();
    let manifest = declared
        .iter()
        .map(|path| {
            format!(
                "[repo.{path}]\npath = \"{path}\"\nurl = \"../{path}.git\"\n"
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(dir.join(".gitvmr"), manifest).unwrap();
    let gitignore =
        ignored.iter().map(|path| format!("/{path}/\n")).collect::<String>();
    fs::write(dir.join(".gitignore"), gitignore).unwrap();
}
