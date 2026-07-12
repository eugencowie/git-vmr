//! Shared unit-test fixtures.

use crate::cli::CliContext;
use crate::git::Git;
use crate::vmr::Repo;
use std::fs;
use std::path::PathBuf;
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

/// Materializes a minimal VMR on disk: a `.gitvmr` marker and two child
/// repos, `backend` and `frontend`.
pub(crate) fn vmr_fixture() -> TempDir
{
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir(tmp.path().join(".gitvmr")).unwrap();
    fs::create_dir_all(tmp.path().join("backend/.git")).unwrap();
    fs::create_dir_all(tmp.path().join("frontend/.git")).unwrap();
    tmp
}
