use anyhow::{Context, Result};
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Repo
{
    pub name: String,
    pub path: PathBuf
}

impl Repo
{
    pub fn from_child_dir(path: PathBuf) -> Result<Option<Repo>>
    {
        if !path.join(".git").exists()
        {
            return Ok(None);
        }

        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .context("repository path has no valid UTF-8 file name")?
            .to_owned();

        Ok(Some(Repo { name, path }))
    }
}
