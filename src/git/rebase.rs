use crate::git::{
    Git, GitCommandResult, command_result, first_non_empty_line_with_fallback
};
use std::path::Path;

impl Git
{
    pub fn rebase(
        &self,
        repo_name: &str,
        repo_path: &Path,
        upstream: &str
    ) -> GitCommandResult
    {
        let output = self.output(repo_path, ["rebase", upstream])?;

        command_result(
            repo_name,
            &output,
            |_| None,
            |output| {
                first_non_empty_line_with_fallback(
                    &output.stderr,
                    &output.stdout,
                    "git rebase failed"
                )
            }
        )
    }
}
