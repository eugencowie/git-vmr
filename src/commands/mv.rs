use crate::cli::CliContext;
use crate::render::{self, Rendered};
use crate::workspace::{MvArgs, Workspace};
use anyhow::Result;

pub fn run(
    workspace: &Workspace,
    context: &CliContext,
    args: MvArgs
) -> Result<Rendered>
{
    let moved = workspace.mv(&context.working_dir, &args)?;
    render::outcomes_in_scope(moved.outcomes, moved.scope_repo_count)
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::cli::Cli;
    use crate::commands::{Command, WorkspaceCommand};
    use clap::error::ErrorKind;
    use std::path::PathBuf;

    #[test]
    fn parses_mv_two_or_more_paths()
    {
        // Act
        let cli =
            Cli::parse_from(["git-vmr", "mv", "one", "two", "three"]).unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Workspace(WorkspaceCommand::Mv(MvArgs {
                sources,
                destination
            })) if sources == [PathBuf::from("one"), PathBuf::from("two")]
                && destination == *"three"
        ));
    }

    #[test]
    fn rejects_mv_without_two_paths_or_with_unsupported_options()
    {
        for args in [
            vec!["git-vmr", "mv"],
            vec!["git-vmr", "mv", "one"],
            vec!["git-vmr", "mv", "-r", "one", "two"],
            vec!["git-vmr", "mv", "--dry-run", "one", "two"],
            vec!["git-vmr", "mv", "-k", "one", "two"],
            vec!["git-vmr", "mv", "--force", "one", "two"]
        ]
        {
            // Act
            let err = Cli::parse_from(args).err().unwrap();

            // Assert
            assert!(
                matches!(
                    err.kind(),
                    ErrorKind::MissingRequiredArgument
                        | ErrorKind::UnknownArgument
                        | ErrorKind::InvalidValue
                        | ErrorKind::TooFewValues
                ),
                "unexpected error kind: {:?}",
                err.kind()
            );
        }
    }
}
