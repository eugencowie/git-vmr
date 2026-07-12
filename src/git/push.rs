use crate::git::{
    Git, GitCommandResult, command_result, first_non_empty_line_with_fallback
};
use std::ffi::OsString;
use std::path::Path;

impl Git
{
    pub fn push(
        &self,
        repo_name: &str,
        repo_path: &Path,
        repository: Option<&str>,
        refspecs: &[String]
    ) -> GitCommandResult
    {
        let mut args = vec![OsString::from("push")];

        if let Some(repository) = repository
        {
            args.push(OsString::from(repository));
        }

        args.extend(refspecs.iter().map(OsString::from));

        let output = self.output(repo_path, args)?;

        command_result(
            repo_name,
            &output,
            |output| {
                let message = first_non_empty_line_with_fallback(
                    &output.stderr,
                    &output.stdout,
                    ""
                );

                if message.is_empty() { None } else { Some(message) }
            },
            |output| {
                first_non_empty_line_with_fallback(
                    &output.stderr,
                    &output.stdout,
                    "git push failed"
                )
            }
        )
    }
}
