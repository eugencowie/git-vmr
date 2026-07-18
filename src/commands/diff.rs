use crate::cli::CliContext;
use crate::render::{self, Failed, Rendered};
use crate::workspace::{AggregatePolicy, Scope, Workspace};
use anyhow::Result;

pub fn run(
    workspace: &Workspace,
    context: &CliContext,
    args: &DiffArgs
) -> Result<Rendered>
{
    // Resolve `auto` against the injected TTY fact here so the diff body
    // only ever sees `always`/`never`
    let color = resolve_color(args.color, context.stdout_is_tty);
    let _paging = paging_active(args.no_pager, context.stdout_is_tty);

    // No paths means the whole VMR; the aggregate path spells the same
    // thing explicitly, matching other path-taking commands
    let scope = if args.paths.is_empty()
    {
        Scope::EntireVmr
    }
    else
    {
        Scope::Paths {
            paths: args.paths.to_vec(),
            aggregate: AggregatePolicy::Allow
        }
    };

    // Diff each owning child repo, gathering its output and its outcome
    let diffs = workspace.map_routed(scope, |git, repo, repo_paths| {
        git.diff(repo, repo_paths, args.staged, color)
    })?;

    // Concatenate in repo order, no separators: repos with an empty diff
    // contribute nothing
    let names =
        diffs.iter().map(|(repo, _)| repo.name.clone()).collect::<Vec<_>>();
    let mut combined = String::new();
    let mut results = Vec::new();
    for (_, (diff, result)) in diffs
    {
        combined.push_str(&diff);
        results.push(result);
    }

    // Successes are quiet, so aggregation renders failures only; the
    // combined diff replaces the (empty) aggregated stdout so surviving
    // diffs still flush when some repos fail
    match render::outcomes(results, names.iter().map(String::as_str))
    {
        Ok(rendered) => Ok(Rendered { stdout: combined, ..rendered }),
        Err(error) => Err(match error.downcast::<Failed>()
        {
            Ok(mut failed) =>
            {
                failed.rendered.stdout = combined;
                render::fail(failed.rendered, failed.message)
            }
            Err(other) => other
        })
    }
}

use crate::git::report::{FailureReport, SuccessReport, command_result};
use crate::git::{Git, GitCommandResult};
use crate::vmr::Repo;
use clap::ValueEnum;
use std::ffi::OsString;
use std::path::PathBuf;

impl Git
{
    /// One child's diff: the raw output for concatenation, paired with the
    /// repo outcome for aggregation.
    fn diff(
        &self,
        repo: &Repo,
        paths: &[PathBuf],
        staged: bool,
        color: ResolvedColor
    ) -> (String, GitCommandResult)
    {
        let mut args = vec![OsString::from("diff")];

        if staged
        {
            args.push(OsString::from("--staged"));
        }

        // Trailing slashes are mandatory: git concatenates the prefix and
        // the repo-relative path verbatim
        args.push(OsString::from(format!("--src-prefix=a/{}/", repo.name)));
        args.push(OsString::from(format!("--dst-prefix=b/{}/", repo.name)));
        args.push(OsString::from(format!("--color={color}")));
        args.push(OsString::from("--no-ext-diff"));
        args.push(OsString::from("--binary"));
        args.push(OsString::from("--"));
        args.extend(paths.iter().map(|path| path.as_os_str().to_owned()));

        let output = match self.output(&repo.path, args)
        {
            Ok(output) => output,
            Err(error) => return (String::new(), Err(error))
        };

        let diff = if output.status.success()
        {
            String::from_utf8_lossy(&output.stdout).into_owned()
        }
        else
        {
            String::new()
        };

        let result = command_result(
            repo,
            &output,
            SuccessReport::quiet(),
            FailureReport::detailed("diff")
        );

        (diff, result)
    }
}

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

impl std::fmt::Display for ResolvedColor
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.write_str(match self
        {
            Self::Always => "always",
            Self::Never => "never"
        })
    }
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

#[cfg(test)]
mod combined_tests
{
    use super::*;
    use crate::git::{Invocation, ScriptedFake};
    use crate::test_support::{cli_context, vmr_fixture};
    use std::ffi::OsString;
    use std::sync::Arc;

    const BACKEND_DIFF: &str = "diff --git a/backend/src/main.rs \
                                b/backend/src/main.rs\n\
                                index 1111111..2222222 100644\n\
                                --- a/backend/src/main.rs\n\
                                +++ b/backend/src/main.rs\n\
                                @@ -1 +1 @@\n\
                                -old\n\
                                +new\n";

    const FRONTEND_DIFF: &str = "diff --git a/frontend/app.rs \
                                 b/frontend/app.rs\n\
                                 index 3333333..4444444 100644\n\
                                 --- a/frontend/app.rs\n\
                                 +++ b/frontend/app.rs\n\
                                 @@ -1 +1 @@\n\
                                 -foo\n\
                                 +bar\n";

    fn diff_args(paths: &[&str]) -> DiffArgs
    {
        DiffArgs {
            staged: false,
            color: ColorWhen::Auto,
            no_pager: false,
            paths: paths.iter().map(PathBuf::from).collect()
        }
    }

    /// The full scripted invocation for one child repo, in the spec's
    /// argument order.
    fn child_args(repo: &str, options: &[&str], paths: &[&str])
    -> Vec<OsString>
    {
        let mut args = vec![OsString::from("diff")];
        args.extend(options.iter().map(OsString::from));
        args.push(OsString::from(format!("--src-prefix=a/{repo}/")));
        args.push(OsString::from(format!("--dst-prefix=b/{repo}/")));
        args.push(OsString::from("--color=never"));
        args.push(OsString::from("--no-ext-diff"));
        args.push(OsString::from("--binary"));
        args.push(OsString::from("--"));
        args.extend(paths.iter().map(OsString::from));
        args
    }

    fn sorted_calls(fake: &ScriptedFake) -> Vec<Invocation>
    {
        let mut calls = fake.calls();
        calls.sort_by(|a, b| a.path.cmp(&b.path));
        calls
    }

    #[test]
    fn combined_diff_concatenates_child_diffs_in_repo_order()
    {
        // Arrange
        let tmp = vmr_fixture();
        let backend = child_args("backend", &[], &["."]);
        let frontend = child_args("frontend", &[], &["."]);
        let fake =
            Arc::new(ScriptedFake::new().on(&backend, 0, BACKEND_DIFF, "").on(
                &frontend,
                0,
                FRONTEND_DIFF,
                ""
            ));
        let git = Git::with(Arc::clone(&fake));
        let workspace = Workspace::find(&git, tmp.path()).unwrap();

        // Act
        let rendered =
            run(&workspace, &cli_context(tmp.path()), &diff_args(&[])).unwrap();

        // Assert: byte-faithful concatenation, no separators
        assert_eq!(rendered.stdout, format!("{BACKEND_DIFF}{FRONTEND_DIFF}"));
        assert!(rendered.stderr.is_empty());

        // Both children were invoked with exactly the spec'd args
        let calls = sorted_calls(&fake);
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].path, tmp.path().join("backend"));
        assert_eq!(calls[0].args, backend);
        assert_eq!(calls[1].path, tmp.path().join("frontend"));
        assert_eq!(calls[1].args, frontend);
    }

    #[test]
    fn staged_flag_and_resolved_color_reach_each_child()
    {
        // Arrange: a TTY resolves `auto` to `always`
        let tmp = vmr_fixture();
        let with_color = |repo: &str| {
            child_args(repo, &["--staged"], &["."])
                .into_iter()
                .map(|arg| {
                    if arg == "--color=never"
                    {
                        OsString::from("--color=always")
                    }
                    else
                    {
                        arg
                    }
                })
                .collect::<Vec<_>>()
        };
        let fake = Arc::new(
            ScriptedFake::new().on(with_color("backend"), 0, "", "").on(
                with_color("frontend"),
                0,
                "",
                ""
            )
        );
        let git = Git::with(Arc::clone(&fake));
        let workspace = Workspace::find(&git, tmp.path()).unwrap();
        let mut context = cli_context(tmp.path());
        context.stdout_is_tty = true;
        let mut args = diff_args(&[]);
        args.staged = true;

        // Act
        run(&workspace, &context, &args).unwrap();

        // Assert
        let calls = sorted_calls(&fake);
        assert_eq!(calls[0].args, with_color("backend"));
        assert_eq!(calls[1].args, with_color("frontend"));
    }

    #[test]
    fn routed_paths_reach_only_the_owning_repo()
    {
        // Arrange
        let tmp = vmr_fixture();
        let backend = child_args("backend", &[], &["src/main.rs"]);
        let fake =
            Arc::new(ScriptedFake::new().on(&backend, 0, BACKEND_DIFF, ""));
        let git = Git::with(Arc::clone(&fake));
        let workspace = Workspace::find(&git, tmp.path()).unwrap();

        // Act
        let rendered = run(
            &workspace,
            &cli_context(tmp.path()),
            &diff_args(&["backend/src/main.rs"])
        )
        .unwrap();

        // Assert
        assert_eq!(rendered.stdout, BACKEND_DIFF);
        let calls = fake.calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].path, tmp.path().join("backend"));
        assert_eq!(calls[0].args, backend);
    }

    #[test]
    fn empty_child_diffs_contribute_nothing()
    {
        // Arrange
        let tmp = vmr_fixture();
        let fake = ScriptedFake::new()
            .on(child_args("backend", &[], &["."]), 0, "", "")
            .on(child_args("frontend", &[], &["."]), 0, FRONTEND_DIFF, "");
        let git = Git::with(fake);
        let workspace = Workspace::find(&git, tmp.path()).unwrap();

        // Act
        let rendered =
            run(&workspace, &cli_context(tmp.path()), &diff_args(&[])).unwrap();

        // Assert
        assert_eq!(rendered.stdout, FRONTEND_DIFF);
    }

    #[test]
    fn failing_repo_reports_while_surviving_diffs_still_print()
    {
        // Arrange
        let tmp = vmr_fixture();
        let fake = ScriptedFake::new()
            .on(child_args("backend", &[], &["."]), 1, "", "fatal: bad\n")
            .on(child_args("frontend", &[], &["."]), 0, FRONTEND_DIFF, "");
        let git = Git::with(fake);
        let workspace = Workspace::find(&git, tmp.path()).unwrap();

        // Act
        let err = run(&workspace, &cli_context(tmp.path()), &diff_args(&[]))
            .unwrap_err();

        // Assert: the surviving diff still flushes, the failure aggregates
        let failed = err.downcast::<Failed>().unwrap();
        assert_eq!(failed.rendered.stdout, FRONTEND_DIFF);
        assert!(failed.message.contains("git diff failed for"));
        assert!(failed.message.contains("fatal: bad"));
        assert!(failed.message.contains("backend"));
    }

    #[test]
    fn unowned_paths_error_like_other_path_taking_commands()
    {
        // Arrange
        let tmp = vmr_fixture();
        let git = Git::with(ScriptedFake::new());
        let workspace = Workspace::find(&git, tmp.path()).unwrap();

        // Act
        let err = run(
            &workspace,
            &cli_context(tmp.path()),
            &diff_args(&["docs/readme.md"])
        )
        .unwrap_err();

        // Assert
        assert!(format!("{err:#}").contains("child Git repository"));
    }
}
