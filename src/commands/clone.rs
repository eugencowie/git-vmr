use crate::git;
use anyhow::Result;
use std::path::Path;

pub fn clone(
    working_dir: &Path,
    repository: &str,
    directory: Option<&Path>
) -> Result<()>
{
    // Clone repository
    git::clone(working_dir, repository, directory)
}
