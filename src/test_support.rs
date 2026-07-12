//! Shared unit-test fixtures.

use std::fs;
use tempfile::TempDir;

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
