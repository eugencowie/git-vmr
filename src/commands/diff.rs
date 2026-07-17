use crate::cli::CliContext;
use crate::render::Rendered;
use crate::workspace::Workspace;
use anyhow::Result;

pub fn run(
    _workspace: &Workspace,
    context: &CliContext,
    args: &DiffArgs
) -> Result<Rendered>
{
    // Resolve `auto` against the injected TTY fact here so the diff body
    // only ever sees `always`/`never`
    let _color = resolve_color(args.color, context.stdout_is_tty);
    let _paging = paging_active(args.no_pager, context.stdout_is_tty);

    // Diff body lands in later tickets
    Ok(String::new().into())
}

use clap::ValueEnum;
use std::path::PathBuf;

/// Show changes between the working trees and index
#[derive(clap::Args)]
pub struct DiffArgs
{
    /// Show changes between the indexes and HEAD
    #[arg(long, visible_alias = "cached")]
    pub staged: bool,

    /// When to colour the output; bare --color means always
    #[arg(
        long,
        value_name = "when",
        value_enum,
        num_args = 0..=1,
        require_equals = true,
        default_value_t = ColorWhen::Auto,
        default_missing_value = "always"
    )]
    pub color: ColorWhen,

    /// Do not pipe the output into a pager
    #[arg(long)]
    pub no_pager: bool,

    /// Limit the diff to the given paths
    #[arg(value_name = "path")]
    pub paths: Vec<PathBuf>
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum ColorWhen
{
    Auto,
    Always,
    Never
}

impl std::fmt::Display for ColorWhen
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.write_str(
            self.to_possible_value().expect("no skipped variants").get_name()
        )
    }
}

/// The colour value children receive: `auto` resolves against the TTY
/// fact, explicit choices pass through.
fn resolve_color(color: ColorWhen, stdout_is_tty: bool) -> ResolvedColor
{
    match color
    {
        ColorWhen::Always => ResolvedColor::Always,
        ColorWhen::Never => ResolvedColor::Never,
        ColorWhen::Auto if stdout_is_tty => ResolvedColor::Always,
        ColorWhen::Auto => ResolvedColor::Never
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ResolvedColor
{
    Always,
    Never
}

/// Paging is active when stdout is a TTY and `--no-pager` is absent —
/// the same TTY fact that gates colour, so they can never disagree.
fn paging_active(no_pager: bool, stdout_is_tty: bool) -> bool
{
    stdout_is_tty && !no_pager
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::cli::Cli;
    use crate::commands::{Command, WorkspaceCommand};

    fn parse(args: &[&str]) -> DiffArgs
    {
        let mut full = vec!["git-vmr", "diff"];
        full.extend(args);
        let cli = Cli::parse_from(full).unwrap();
        match cli.command
        {
            Command::Workspace(WorkspaceCommand::Diff(args)) => args,
            _ => panic!("expected the diff subcommand")
        }
    }

    #[test]
    fn color_defaults_to_auto()
    {
        assert_eq!(parse(&[]).color, ColorWhen::Auto);
    }

    #[test]
    fn bare_color_flag_means_always()
    {
        assert_eq!(parse(&["--color"]).color, ColorWhen::Always);
    }

    #[test]
    fn color_accepts_explicit_when_with_equals()
    {
        assert_eq!(parse(&["--color=never"]).color, ColorWhen::Never);
    }

    #[test]
    fn cached_is_an_alias_for_staged()
    {
        assert!(parse(&["--cached"]).staged);
    }

    #[test]
    fn staged_and_no_pager_flags_parse()
    {
        let args = parse(&["--staged", "--no-pager"]);
        assert!(args.staged);
        assert!(args.no_pager);
    }

    #[test]
    fn trailing_paths_parse_with_and_without_separator()
    {
        let args = parse(&["backend/src"]);
        assert_eq!(args.paths, [PathBuf::from("backend/src")]);

        let args = parse(&["--", "backend/src"]);
        assert_eq!(args.paths, [PathBuf::from("backend/src")]);
    }

    #[test]
    fn auto_resolves_to_always_on_a_tty()
    {
        assert_eq!(resolve_color(ColorWhen::Auto, true), ResolvedColor::Always);
    }

    #[test]
    fn auto_resolves_to_never_off_a_tty()
    {
        assert_eq!(resolve_color(ColorWhen::Auto, false), ResolvedColor::Never);
    }

    #[test]
    fn explicit_choices_pass_through_regardless_of_tty()
    {
        assert_eq!(
            resolve_color(ColorWhen::Always, false),
            ResolvedColor::Always
        );
        assert_eq!(resolve_color(ColorWhen::Never, true), ResolvedColor::Never);
    }

    #[test]
    fn paging_is_active_only_on_a_tty_without_no_pager()
    {
        assert!(paging_active(false, true));
        assert!(!paging_active(true, true));
        assert!(!paging_active(false, false));
        assert!(!paging_active(true, false));
    }
}
