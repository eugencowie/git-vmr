use crate::git::Git;
use anyhow::Result;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;

/// Clone a repository into a new directory
#[derive(clap::Args)]
pub struct CloneArgs
{
    /// The (possibly remote) <repository> to clone from
    #[arg(value_name = "repository")]
    pub repository: String,

    /// The name of a new directory to clone into
    #[arg(value_name = "directory")]
    pub directory: Option<PathBuf>
}

impl Git
{
    /// Runs git clone with inherited stdio so credential prompts and
    /// progress reach the user. Git reports its own errors on stderr; the
    /// caller decides what a non-success exit status means.
    pub fn clone(
        &self,
        working_dir: &Path,
        repository: &str,
        directory: Option<&Path>
    ) -> Result<ExitStatus>
    {
        let mut args =
            vec![OsString::from("clone"), OsString::from(repository)];

        if let Some(directory) = directory
        {
            args.push(directory.as_os_str().to_owned());
        }

        self.runner.run_interactive(working_dir, &args)
    }
}
