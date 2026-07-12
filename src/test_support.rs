//! Shared unit-test fixtures.

use crate::vmr::Repo;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// A child repo value for exercising git ops directly.
pub(crate) fn repo(name: &str, path: &str) -> Repo
{
    Repo { name: name.to_owned(), path: PathBuf::from(path) }
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
