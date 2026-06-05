use crate::git::{GitCommandResult, command_result, git_path_output, stderr};
use anyhow::{Result, bail};
use std::ffi::OsString;
use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChmodMode
{
    Executable,
    NotExecutable
}

impl ChmodMode
{
    fn as_git_value(self) -> &'static str
    {
        match self
        {
            ChmodMode::Executable => "+x",
            ChmodMode::NotExecutable => "-x"
        }
    }
}

impl fmt::Display for ChmodMode
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        f.write_str(self.as_git_value())
    }
}

impl FromStr for ChmodMode
{
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err>
    {
        match value
        {
            "+x" => Ok(ChmodMode::Executable),
            "-x" => Ok(ChmodMode::NotExecutable),
            _ => Err("expected +x or -x".to_owned())
        }
    }
}

pub fn add(
    repo_name: &str,
    repo_path: &Path,
    paths: &[PathBuf],
    all: bool,
    force: bool,
    chmod: Option<ChmodMode>
) -> GitCommandResult
{
    let output =
        git_path_output(repo_path, add_args(all, force, chmod), paths)?;

    command_result(
        repo_name,
        &output,
        |_| None,
        |output| {
            format!(
                "git add failed for '{}': {}",
                repo_path.display(),
                stderr(output)
            )
        }
    )
}

fn add_args(all: bool, force: bool, chmod: Option<ChmodMode>) -> Vec<OsString>
{
    let mut args = vec![OsString::from("add")];

    if all
    {
        args.push(OsString::from("--all"));
    }

    if force
    {
        args.push(OsString::from("--force"));
    }

    if let Some(chmod) = chmod
    {
        args.push(OsString::from(format!("--chmod={chmod}")));
    }

    args.push(OsString::from("--"));
    args
}

pub fn add_path(repo_path: &Path, path: &Path) -> Result<()>
{
    let output = git_path_output(repo_path, ["add", "--"], [path])?;

    if !output.status.success()
    {
        bail!(
            "git add failed for '{}': {}",
            repo_path.display(),
            stderr(&output)
        );
    }

    Ok(())
}
