use clap::parser::ValueSource;
use clap::{ArgMatches, Command};

/// Event metadata: the record of what was invoked — the dotted command name
/// and the names of flags explicitly given on the command line. Names only,
/// never values; positionals excluded.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CliMetadata
{
    pub name: String,
    pub flags: Vec<String>,
    pub global_flags: Vec<String>
}

/// Derive event metadata from the CLI definition and its parsed matches.
pub fn event_meta(command: &Command, matches: &ArgMatches) -> CliMetadata
{
    let mut meta = CliMetadata {
        name: String::new(),
        flags: Vec::new(),
        global_flags: present_flags(command, matches)
    };

    let mut level = (command, matches);
    while let Some((subcommand_name, submatches)) = level.1.subcommand()
    {
        let Some(subcommand) = level.0.find_subcommand(subcommand_name)
        else
        {
            break;
        };

        if !meta.name.is_empty()
        {
            meta.name.push('.');
        }
        meta.name.push_str(subcommand_name);
        meta.flags.extend(present_flags(subcommand, submatches));
        level = (subcommand, submatches);
    }

    meta
}

/// Names of the non-positional args explicitly given on the command line.
fn present_flags(command: &Command, matches: &ArgMatches) -> Vec<String>
{
    command
        .get_arguments()
        .filter(|arg| !arg.is_positional())
        .filter(|arg| {
            matches.value_source(arg.get_id().as_str())
                == Some(ValueSource::CommandLine)
        })
        .map(|arg| arg.get_id().to_string())
        .collect()
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::cli::Cli;
    use clap::CommandFactory;

    fn meta(args: &[&str]) -> CliMetadata
    {
        let mut command = Cli::command();
        let matches = command
            .try_get_matches_from_mut(args)
            .expect("failed to parse test arguments");
        event_meta(&command, &matches)
    }

    #[test]
    fn records_value_flags_without_values()
    {
        let m = meta(&["git-vmr", "-C", "/tmp", "add", "--chmod=+x", "src.rs"]);

        assert_eq!(m.name, "add");
        assert_eq!(m.flags, ["chmod"]);
        assert_eq!(m.global_flags, ["working_dir"]);
    }

    #[test]
    fn records_nested_subcommands_with_dotted_names()
    {
        let m = meta(&[
            "git-vmr", "worktree", "remove", "--force", "--delete", "../feat"
        ]);

        assert_eq!(m.name, "worktree.remove");
        assert_eq!(m.flags, ["force", "delete"]);
        assert_eq!(m.global_flags, Vec::<String>::new());
    }

    #[test]
    fn records_flag_but_not_trailing_values()
    {
        let m = meta(&["git-vmr", "foreach", "--quiet", "echo", "secret"]);

        assert_eq!(m.name, "foreach");
        assert_eq!(m.flags, ["quiet"]);
    }

    #[test]
    fn records_count_flag_only_when_given()
    {
        let given = meta(&["git-vmr", "worktree", "move", "-f", "a", "b"]);
        let omitted = meta(&["git-vmr", "worktree", "move", "a", "b"]);

        assert_eq!(given.flags, ["force"]);
        assert_eq!(omitted.flags, Vec::<String>::new());
    }

    #[test]
    fn records_required_option_as_present()
    {
        let m = meta(&["git-vmr", "commit", "-m", "message text"]);

        assert_eq!(m.name, "commit");
        assert_eq!(m.flags, ["message"]);
    }

    #[test]
    fn excludes_positional_arguments()
    {
        let m = meta(&["git-vmr", "fetch", "origin", "main"]);

        assert_eq!(m.name, "fetch");
        assert_eq!(m.flags, Vec::<String>::new());
    }

    #[test]
    fn records_defaulted_subcommand_as_invoked()
    {
        assert_eq!(meta(&["git-vmr", "worktree"]).name, "worktree");
        assert_eq!(
            meta(&["git-vmr", "worktree", "list"]).name,
            "worktree.list"
        );
    }

    #[test]
    fn records_subcommand_alias_under_canonical_name()
    {
        let m = meta(&["git-vmr", "worktree", "rm", "../feat"]);

        assert_eq!(m.name, "worktree.remove");
    }
}
