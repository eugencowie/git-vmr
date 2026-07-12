use crate::git::Git;
use anyhow::Result;
use std::ffi::OsString;
use std::path::Path;
use std::process::ExitStatus;

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
