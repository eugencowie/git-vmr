use crate::git::report::{FailureReport, SuccessReport, command_result};
use crate::git::{Git, GitCommandResult};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

impl Git
{
    pub fn restore(
        &self,
        repo_name: &str,
        repo_path: &Path,
        paths: &[PathBuf],
        worktree: bool,
        staged: bool
    ) -> GitCommandResult
    {
        let mut args = vec![OsString::from("restore")];

        if staged
        {
            args.push(OsString::from("--staged"));
        }

        if worktree
        {
            args.push(OsString::from("--worktree"));
        }

        args.push(OsString::from("--"));
        args.extend(paths.iter().map(|path| path.as_os_str().to_owned()));
        let output = self.output(repo_path, args)?;

        command_result(
            repo_name,
            repo_path,
            &output,
            SuccessReport::quiet(),
            FailureReport::detailed("restore")
        )
    }
}
