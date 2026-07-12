use crate::git::{
    Git, GitCommandResult, command_result, first_non_empty_line,
    first_non_empty_line_with_fallback
};
use std::path::Path;

impl Git
{
    pub fn merge(
        &self,
        repo_name: &str,
        repo_path: &Path,
        commit_ish: &str
    ) -> GitCommandResult
    {
        let output = self.output(repo_path, ["merge", commit_ish])?;

        command_result(
            repo_name,
            &output,
            |output| {
                Some(first_non_empty_line(
                    &output.stdout,
                    "git merge succeeded"
                ))
            },
            |output| {
                first_non_empty_line_with_fallback(
                    &output.stderr,
                    &output.stdout,
                    "git merge failed"
                )
            }
        )
    }
}
