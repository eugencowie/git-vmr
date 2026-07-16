use crate::git::report::{FailureReport, SuccessReport, command_result};
use crate::git::{Git, GitCommandResult, stderr};
use crate::vmr::Repo;
use anyhow::{Result, bail};
use std::ffi::OsString;
use std::fmt::{Display, Formatter};
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

impl Display for ChmodMode
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
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

/// Add file contents to the index
#[derive(clap::Args)]
pub struct AddArgs
{
    #[command(flatten)]
    pub options: AddOptions,

    /// Files to add content from
    #[arg(required_unless_present = "all", num_args = 0.., value_name = "pathspec")]
    pub paths: Vec<PathBuf>
}

#[derive(clap::Args)]
pub struct AddOptions
{
    /// Allow adding otherwise ignored files
    #[arg(short, long)]
    pub force: bool,

    /// Update the index not only where the working tree has a file
    /// matching [pathspec] but also where the index already has an
    /// entry
    #[arg(short = 'A', long)]
    pub all: bool,

    /// Override the executable bit of added files
    #[arg(long, value_name = "(+|-)x")]
    pub chmod: Option<ChmodMode>
}

fn add_args(options: &AddOptions) -> Vec<OsString>
{
    let mut args = vec![OsString::from("add")];

    if options.all
    {
        args.push(OsString::from("--all"));
    }

    if options.force
    {
        args.push(OsString::from("--force"));
    }

    if let Some(chmod) = options.chmod
    {
        args.push(OsString::from(format!("--chmod={chmod}")));
    }

    args.push(OsString::from("--"));
    args
}

impl Git
{
    pub fn add(
        &self,
        repo: &Repo,
        paths: &[PathBuf],
        options: &AddOptions
    ) -> GitCommandResult
    {
        let output = self.path_output(&repo.path, add_args(options), paths)?;

        command_result(
            repo,
            &output,
            SuccessReport::quiet(),
            FailureReport::detailed("add")
        )
    }

    pub fn add_path(&self, repo_path: &Path, path: &Path) -> Result<()>
    {
        let output = self.path_output(repo_path, ["add", "--"], [path])?;

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
}
