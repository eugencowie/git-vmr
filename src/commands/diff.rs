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
    let color = resolve_color(
        args.color,
        context.stdout_is_tty,
        context.stdout_supports_color
    );
    let pager = if paging_active(args.no_pager, context.stdout_is_tty)
    {
        resolve_pager(workspace)
    }
    else
    {
        None
    };

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
        Ok(rendered) => Ok(Rendered { stdout: combined, pager, ..rendered }),
        Err(error) => Err(match error.downcast::<Failed>()
        {
            Ok(mut failed) =>
            {
                failed.rendered.stdout = combined;
                failed.rendered.pager = pager;
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
        args.push(OsString::from("--no-textconv"));
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
            let stdout = match std::str::from_utf8(&output.stdout)
            {
                Ok(stdout) => stdout,
                Err(error) =>
                    return (
                        String::new(),
                        Err(anyhow::anyhow!(
                            "git diff produced non-UTF-8 output for '{}': {error}",
                            repo.path.display()
                        ))
                    ),
            };
            rewrite_rename_headers(stdout, &repo.name)
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

/// Git never prefixes `rename from`/`rename to`/`copy from`/`copy to`
/// lines (`--src-prefix`/`--dst-prefix` do not apply to them), so this
/// pure transform prepends `<repo>/` — no `a/`/`b/` — to the path on
/// those lines. Without it `git apply -p1` from the VMR root rejects
/// rename patches.
///
/// Only lines inside a file block's extended headers are rewritten: from
/// a `diff --git` line to the first `@@`, or to the next `diff --git` /
/// end of input when the block has no hunks (pure 100% renames). When the
/// path token is C-quoted the prefix goes inside the quotes, immediately
/// after the opening `"`; the prefix needs no escaping, so the quoted
/// bytes are otherwise kept verbatim. SGR colour codes wrap header lines
/// outside the text and are skipped, not touched.
fn rewrite_rename_headers(diff: &str, repo: &str) -> String
{
    const KEYWORDS: [&str; 4] =
        ["rename from ", "rename to ", "copy from ", "copy to "];

    let mut out = String::with_capacity(diff.len());
    let mut in_headers = false;

    for line in diff.split_inclusive('\n')
    {
        let text = &line[after_leading_sgr(line)..];

        if text.starts_with("diff --git ")
        {
            in_headers = true;
        }
        else if text.starts_with("@@")
        {
            in_headers = false;
        }
        else if in_headers
            && let Some(keyword) =
                KEYWORDS.iter().find(|keyword| text.starts_with(**keyword))
        {
            // Split at the start of the path token; a quoted token gets
            // the prefix just inside the opening quote, a bare one at
            // the start
            let mut path_start = (line.len() - text.len()) + keyword.len();
            if line[path_start..].starts_with('"')
            {
                path_start += 1;
            }
            out.push_str(&line[..path_start]);
            out.push_str(repo);
            out.push('/');
            out.push_str(&line[path_start..]);
            continue;
        }

        out.push_str(line);
    }

    out
}

/// Byte index just past any leading SGR escape sequences
/// (`ESC [ ... m`), so coloured header lines match the same prefixes as
/// plain ones.
fn after_leading_sgr(line: &str) -> usize
{
    let bytes = line.as_bytes();
    let mut index = 0;
    while bytes.get(index) == Some(&0x1b) && bytes.get(index + 1) == Some(&b'[')
    {
        match bytes[index + 2..].iter().position(|byte| *byte == b'm')
        {
            Some(end) => index += 2 + end + 1,
            None => break
        }
    }
    index
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
/// fact and the VT capability (a legacy conhost would print unrendered
/// escapes), explicit choices pass through. VT gates auto-colour only —
/// paging still gates on the TTY fact alone.
fn resolve_color(
    color: ColorWhen,
    stdout_is_tty: bool,
    stdout_supports_color: bool
) -> ResolvedColor
{
    match color
    {
        ColorWhen::Always => ResolvedColor::Always,
        ColorWhen::Never => ResolvedColor::Never,
        ColorWhen::Auto if stdout_is_tty && stdout_supports_color =>
            ResolvedColor::Always,
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

/// The pager argv to hand to emit: `[<shell>, "-c", <pager command>]`.
/// The pager command is one `git var GIT_PAGER` from the VMR root, so
/// git's full `GIT_PAGER` → `core.pager` → `PAGER` → `less` chain applies
/// and a child repo's local `core.pager` cannot hijack the combined view.
/// `cat`, an empty resolution, or a failed lookup all mean no paging. The
/// shell is resolved first-party too — `git var GIT_SHELL_PATH`,
/// uniformly on all platforms; when that fails (git predates 2.45) the
/// literal `sh` preserves today's behaviour.
fn resolve_pager(workspace: &Workspace) -> Option<Vec<String>>
{
    let pager = git_var(workspace, "GIT_PAGER")?;
    if pager == "cat"
    {
        return None;
    }

    let shell =
        git_var(workspace, "GIT_SHELL_PATH").unwrap_or_else(|| "sh".to_owned());
    Some(vec![shell, "-c".to_owned(), pager])
}

/// One `git var` lookup from the VMR root; a failed lookup or an empty
/// value is `None`.
fn git_var(workspace: &Workspace, name: &str) -> Option<String>
{
    let output =
        workspace.git().output(workspace.root(), ["var", name]).ok()?;
    if !output.status.success()
    {
        return None;
    }

    let value = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    (!value.is_empty()).then_some(value)
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
    fn auto_resolves_to_always_on_a_vt_capable_tty()
    {
        assert_eq!(
            resolve_color(ColorWhen::Auto, true, true),
            ResolvedColor::Always
        );
    }

    #[test]
    fn auto_resolves_to_never_off_a_tty()
    {
        assert_eq!(
            resolve_color(ColorWhen::Auto, false, true),
            ResolvedColor::Never
        );
    }

    #[test]
    fn auto_resolves_to_never_on_a_tty_without_vt_output()
    {
        // Legacy conhost: a real console that cannot render escapes
        assert_eq!(
            resolve_color(ColorWhen::Auto, true, false),
            ResolvedColor::Never
        );
    }

    #[test]
    fn explicit_choices_pass_through_regardless_of_tty_and_vt()
    {
        assert_eq!(
            resolve_color(ColorWhen::Always, false, false),
            ResolvedColor::Always
        );
        assert_eq!(
            resolve_color(ColorWhen::Never, true, true),
            ResolvedColor::Never
        );
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
mod rewrite_tests
{
    use super::rewrite_rename_headers;

    #[test]
    fn rename_with_hunks_rewrites_headers_only()
    {
        let input = "diff --git a/backend/old.rs b/backend/new.rs\n\
                     similarity index 90%\n\
                     rename from old.rs\n\
                     rename to new.rs\n\
                     index 1111111..2222222 100644\n\
                     --- a/backend/old.rs\n\
                     +++ b/backend/new.rs\n\
                     @@ -1 +1 @@\n\
                     -old\n\
                     +new\n";
        let expected = "diff --git a/backend/old.rs b/backend/new.rs\n\
                        similarity index 90%\n\
                        rename from backend/old.rs\n\
                        rename to backend/new.rs\n\
                        index 1111111..2222222 100644\n\
                        --- a/backend/old.rs\n\
                        +++ b/backend/new.rs\n\
                        @@ -1 +1 @@\n\
                        -old\n\
                        +new\n";
        assert_eq!(rewrite_rename_headers(input, "backend"), expected);
    }

    #[test]
    fn pure_rename_without_hunks_is_rewritten()
    {
        let input = "diff --git a/backend/old.rs b/backend/new.rs\n\
                     similarity index 100%\n\
                     rename from old.rs\n\
                     rename to new.rs\n";
        let expected = "diff --git a/backend/old.rs b/backend/new.rs\n\
                        similarity index 100%\n\
                        rename from backend/old.rs\n\
                        rename to backend/new.rs\n";
        assert_eq!(rewrite_rename_headers(input, "backend"), expected);
    }

    #[test]
    fn copy_lines_are_rewritten()
    {
        let input = "diff --git a/backend/src.txt b/backend/dest.txt\n\
                     similarity index 100%\n\
                     copy from src.txt\n\
                     copy to dest.txt\n";
        let expected = "diff --git a/backend/src.txt b/backend/dest.txt\n\
                        similarity index 100%\n\
                        copy from backend/src.txt\n\
                        copy to backend/dest.txt\n";
        assert_eq!(rewrite_rename_headers(input, "backend"), expected);
    }

    #[test]
    fn rename_from_inside_a_hunk_body_is_untouched()
    {
        let input = "diff --git a/backend/notes.txt b/backend/notes.txt\n\
                     index 1111111..2222222 100644\n\
                     --- a/backend/notes.txt\n\
                     +++ b/backend/notes.txt\n\
                     @@ -1 +1,2 @@\n\
                     rename from old.rs\n\
                     +rename to new.rs\n";
        assert_eq!(rewrite_rename_headers(input, "backend"), input);
    }

    #[test]
    fn coloured_rename_lines_are_rewritten_inside_the_sgr_wrapping()
    {
        let input = "\x1b[1mdiff --git a/backend/old.rs \
                     b/backend/new.rs\x1b[m\n\
                     \x1b[1msimilarity index 100%\x1b[m\n\
                     \x1b[1mrename from old.rs\x1b[m\n\
                     \x1b[1mrename to new.rs\x1b[m\n";
        let expected = "\x1b[1mdiff --git a/backend/old.rs \
                        b/backend/new.rs\x1b[m\n\
                        \x1b[1msimilarity index 100%\x1b[m\n\
                        \x1b[1mrename from backend/old.rs\x1b[m\n\
                        \x1b[1mrename to backend/new.rs\x1b[m\n";
        assert_eq!(rewrite_rename_headers(input, "backend"), expected);
    }

    #[test]
    fn multi_file_mixed_patch_rewrites_each_rename_block()
    {
        let input = "diff --git a/backend/kept.rs b/backend/kept.rs\n\
                     index 1111111..2222222 100644\n\
                     --- a/backend/kept.rs\n\
                     +++ b/backend/kept.rs\n\
                     @@ -1 +1 @@\n\
                     -foo\n\
                     +bar\n\
                     diff --git a/backend/old.rs b/backend/new.rs\n\
                     similarity index 100%\n\
                     rename from old.rs\n\
                     rename to new.rs\n\
                     diff --git a/backend/other.rs b/backend/other.rs\n\
                     index 3333333..4444444 100644\n\
                     --- a/backend/other.rs\n\
                     +++ b/backend/other.rs\n\
                     @@ -1 +1 @@\n\
                     -baz\n\
                     +qux\n";
        let expected = "diff --git a/backend/kept.rs b/backend/kept.rs\n\
                        index 1111111..2222222 100644\n\
                        --- a/backend/kept.rs\n\
                        +++ b/backend/kept.rs\n\
                        @@ -1 +1 @@\n\
                        -foo\n\
                        +bar\n\
                        diff --git a/backend/old.rs b/backend/new.rs\n\
                        similarity index 100%\n\
                        rename from backend/old.rs\n\
                        rename to backend/new.rs\n\
                        diff --git a/backend/other.rs b/backend/other.rs\n\
                        index 3333333..4444444 100644\n\
                        --- a/backend/other.rs\n\
                        +++ b/backend/other.rs\n\
                        @@ -1 +1 @@\n\
                        -baz\n\
                        +qux\n";
        assert_eq!(rewrite_rename_headers(input, "backend"), expected);
    }

    // Bytes lifted verbatim from research/quoted-paths.md (git 2.53,
    // `--src-prefix=a/backend/ --dst-prefix=b/backend/`): the prefix goes
    // inside the quotes, immediately after the opening `"`, with no
    // re-escaping of the existing quoted bytes
    #[test]
    fn quoted_path_rename_gets_the_prefix_inside_the_quotes()
    {
        let input = concat!(
            r#"diff --git "a/backend/quote\"file.txt" "#,
            r#""b/backend/newquote\"file.txt""#,
            "\n",
            "similarity index 100%\n",
            r#"rename from "quote\"file.txt""#,
            "\n",
            r#"rename to "newquote\"file.txt""#,
            "\n"
        );
        let expected = concat!(
            r#"diff --git "a/backend/quote\"file.txt" "#,
            r#""b/backend/newquote\"file.txt""#,
            "\n",
            "similarity index 100%\n",
            r#"rename from "backend/quote\"file.txt""#,
            "\n",
            r#"rename to "backend/newquote\"file.txt""#,
            "\n"
        );
        assert_eq!(rewrite_rename_headers(input, "backend"), expected);
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
        args.push(OsString::from("--no-textconv"));
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
            ScriptedFake::new()
                .on(with_color("backend"), 0, "", "")
                .on(with_color("frontend"), 0, "", "")
                .on(["var", "GIT_PAGER"], 0, "cat\n", "")
        );
        let git = Git::with(Arc::clone(&fake));
        let workspace = Workspace::find(&git, tmp.path()).unwrap();
        let mut context = cli_context(tmp.path());
        context.stdout_is_tty = true;
        let mut args = diff_args(&[]);
        args.staged = true;

        // Act
        run(&workspace, &context, &args).unwrap();

        // Assert: the pager lookup runs at the VMR root, the children get
        // the resolved flags
        let calls = sorted_calls(&fake);
        assert_eq!(calls[0].path, tmp.path());
        assert_eq!(calls[1].args, with_color("backend"));
        assert_eq!(calls[2].args, with_color("frontend"));
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
    fn non_utf8_diff_output_is_rejected_without_lossy_substitution()
    {
        let tmp = vmr_fixture();
        let backend = child_args("backend", &[], &["src/main.rs"]);
        let fake = ScriptedFake::new().on_bytes(
            &backend,
            0,
            b"diff --git a/backend/file b/backend/file\n+\xff\n",
            b""
        );
        let git = Git::with(fake);
        let workspace = Workspace::find(&git, tmp.path()).unwrap();

        let error = run(
            &workspace,
            &cli_context(tmp.path()),
            &diff_args(&["backend/src/main.rs"])
        )
        .unwrap_err();

        let failed = error.downcast::<Failed>().unwrap();
        assert!(failed.rendered.stdout.is_empty());
        assert!(failed.message.contains("non-UTF-8 output"));
        assert!(failed.message.contains("backend"));
        assert!(!failed.message.contains('\u{fffd}'));
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

#[cfg(test)]
mod pager_tests
{
    use super::*;
    use crate::git::ScriptedFake;
    use crate::test_support::{cli_context, vmr_fixture};
    use std::sync::Arc;

    fn empty_diffs() -> ScriptedFake
    {
        let child = |repo: &str| {
            [
                "diff".to_owned(),
                format!("--src-prefix=a/{repo}/"),
                format!("--dst-prefix=b/{repo}/"),
                "--color=always".to_owned(),
                "--no-ext-diff".to_owned(),
                "--no-textconv".to_owned(),
                "--binary".to_owned(),
                "--".to_owned(),
                ".".to_owned()
            ]
        };
        ScriptedFake::new().on(child("backend"), 0, "", "").on(
            child("frontend"),
            0,
            "",
            ""
        )
    }

    fn run_with_tty(fake: ScriptedFake, no_pager: bool) -> Rendered
    {
        let tmp = vmr_fixture();
        let git = Git::with(fake);
        let workspace = Workspace::find(&git, tmp.path()).unwrap();
        let mut context = cli_context(tmp.path());
        context.stdout_is_tty = true;
        let args = DiffArgs {
            staged: false,
            color: ColorWhen::Auto,
            no_pager,
            paths: Vec::new()
        };
        run(&workspace, &context, &args).unwrap()
    }

    #[test]
    fn tty_resolves_the_pager_and_shell_through_git_var()
    {
        let fake = empty_diffs().on(["var", "GIT_PAGER"], 0, "less\n", "").on(
            ["var", "GIT_SHELL_PATH"],
            0,
            "/usr/bin/sh\n",
            ""
        );

        let rendered = run_with_tty(fake, false);

        assert_eq!(
            rendered.pager,
            Some(vec![
                "/usr/bin/sh".to_owned(),
                "-c".to_owned(),
                "less".to_owned()
            ])
        );
    }

    #[test]
    fn failed_shell_resolution_falls_back_to_literal_sh()
    {
        // git < 2.45 has no GIT_SHELL_PATH; the lookup fails and the
        // literal `sh` preserves today's behaviour
        let fake = empty_diffs().on(["var", "GIT_PAGER"], 0, "less\n", "").on(
            ["var", "GIT_SHELL_PATH"],
            1,
            "",
            "fatal: unknown\n"
        );

        let rendered = run_with_tty(fake, false);

        assert_eq!(
            rendered.pager,
            Some(vec!["sh".to_owned(), "-c".to_owned(), "less".to_owned()])
        );
    }

    #[test]
    fn cat_and_empty_resolutions_mean_no_paging()
    {
        for resolution in ["cat\n", ""]
        {
            let fake =
                empty_diffs().on(["var", "GIT_PAGER"], 0, resolution, "");

            assert_eq!(run_with_tty(fake, false).pager, None);
        }
    }

    #[test]
    fn failed_resolution_means_no_paging()
    {
        let fake = empty_diffs().on(["var", "GIT_PAGER"], 1, "", "boom\n");

        assert_eq!(run_with_tty(fake, false).pager, None);
    }

    #[test]
    fn no_pager_flag_skips_resolution_entirely()
    {
        // The scripted fake panics on an unscripted `var GIT_PAGER`, so
        // completing without one proves the lookup never ran
        assert_eq!(run_with_tty(empty_diffs(), true).pager, None);
    }

    #[test]
    fn piped_stdout_skips_resolution_entirely()
    {
        let tmp = vmr_fixture();
        let child = |repo: &str| {
            [
                "diff".to_owned(),
                format!("--src-prefix=a/{repo}/"),
                format!("--dst-prefix=b/{repo}/"),
                "--color=never".to_owned(),
                "--no-ext-diff".to_owned(),
                "--no-textconv".to_owned(),
                "--binary".to_owned(),
                "--".to_owned(),
                ".".to_owned()
            ]
        };
        let git =
            Git::with(ScriptedFake::new().on(child("backend"), 0, "", "").on(
                child("frontend"),
                0,
                "",
                ""
            ));
        let workspace = Workspace::find(&git, tmp.path()).unwrap();
        let args = DiffArgs {
            staged: false,
            color: ColorWhen::Auto,
            no_pager: false,
            paths: Vec::new()
        };

        let rendered =
            run(&workspace, &cli_context(tmp.path()), &args).unwrap();

        assert_eq!(rendered.pager, None);
    }

    #[test]
    fn failure_output_still_carries_the_pager()
    {
        let fake = Arc::new(
            empty_diffs()
                .on(["var", "GIT_PAGER"], 0, "less\n", "")
                .on(["var", "GIT_SHELL_PATH"], 0, "/usr/bin/sh\n", "")
                .on(
                    [
                        "diff",
                        "--src-prefix=a/backend/",
                        "--dst-prefix=b/backend/",
                        "--color=always",
                        "--no-ext-diff",
                        "--no-textconv",
                        "--binary",
                        "--",
                        "src/main.rs"
                    ],
                    1,
                    "",
                    "fatal: bad\n"
                )
        );
        let tmp = vmr_fixture();
        let git = Git::with(Arc::clone(&fake));
        let workspace = Workspace::find(&git, tmp.path()).unwrap();
        let mut context = cli_context(tmp.path());
        context.stdout_is_tty = true;
        let args = DiffArgs {
            staged: false,
            color: ColorWhen::Auto,
            no_pager: false,
            paths: vec![PathBuf::from("backend/src/main.rs")]
        };

        let err = run(&workspace, &context, &args).unwrap_err();

        let failed = err.downcast::<Failed>().unwrap();
        assert_eq!(
            failed.rendered.pager,
            Some(vec![
                "/usr/bin/sh".to_owned(),
                "-c".to_owned(),
                "less".to_owned()
            ])
        );
    }
}
