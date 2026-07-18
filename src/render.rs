//! Rendering: turning per-repo data and outcomes into styled terminal text.
//! Rendering is pure — printing happens once, at the [`emit`] choke point.

mod outcomes;

use anstyle::{AnsiColor, Color, Style};
use anyhow::Result;
pub use outcomes::outcomes;
use std::fmt;

/// Rendered terminal text for one command, per stream. Commands return this
/// instead of printing.
#[derive(Debug, Default)]
pub struct Rendered
{
    pub stdout: String,
    pub stderr: String,
    /// The argv to page `stdout` through; `None` means print directly.
    pub pager: Option<Vec<String>>
}

impl From<String> for Rendered
{
    fn from(stdout: String) -> Self
    {
        Self { stdout, ..Self::default() }
    }
}

/// A failed command that still has rendered output to flush — for example
/// per-repo successes reported alongside grouped failures.
#[derive(Debug)]
pub struct Failed
{
    pub rendered: Rendered,
    pub message: String
}

impl fmt::Display for Failed
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Failed {}

/// Wraps rendered output and a failure message into a command error.
pub(crate) fn fail(rendered: Rendered, message: String) -> anyhow::Error
{
    anyhow::Error::new(Failed { rendered, message })
}

/// The single print choke point: flushes rendered output to the right
/// streams and reduces the command result to pass or fail.
pub fn emit(result: Result<Rendered>) -> Result<()>
{
    match result
    {
        Ok(rendered) =>
        {
            print(&rendered);
            Ok(())
        }
        Err(error) => Err(match error.downcast::<Failed>()
        {
            Ok(failed) =>
            {
                print(&failed.rendered);
                anyhow::Error::msg(failed.message)
            }
            Err(other) => other
        })
    }
}

fn print(rendered: &Rendered)
{
    match &rendered.pager
    {
        Some(pager) if !rendered.stdout.is_empty() =>
            page(pager, &rendered.stdout),
        Some(_) | None => anstream::print!("{}", rendered.stdout)
    }
    anstream::eprint!("{}", rendered.stderr);
}

/// Pipes text through the pager argv, git-style: `argv[0]` is spawned
/// with the remaining args verbatim (emit knows nothing of shells or
/// `-c`), `dirname(argv[0])` is prepended to the child's `PATH`
/// (mirrors git's private-PATH augmentation so Git for Windows' `less`
/// resolves beside its `sh`; harmless on Unix), and `LESS`/`LV` get
/// git's defaults when unset. A pager that dies loses the text — no
/// re-print on nonzero pager exit. Returns only once the pager exits, so
/// stderr always lands after the paged view closes. When `argv[0]` itself
/// cannot spawn the text is printed directly, silently — the designed
/// degradation, identical to `--no-pager` output.
fn page(argv: &[String], text: &str)
{
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut command = Command::new(&argv[0]);
    command.args(&argv[1..]).stdin(Stdio::piped());
    if let Some(dir) = std::path::Path::new(&argv[0])
        .parent()
        .filter(|dir| !dir.as_os_str().is_empty())
    {
        let path = std::env::var_os("PATH").unwrap_or_default();
        let paths =
            std::iter::once(dir.to_owned()).chain(std::env::split_paths(&path));
        if let Ok(joined) = std::env::join_paths(paths)
        {
            command.env("PATH", joined);
        }
    }
    if std::env::var_os("LESS").is_none()
    {
        command.env("LESS", "FRX");
    }
    if std::env::var_os("LV").is_none()
    {
        command.env("LV", "-c");
    }

    let Ok(mut child) = command.spawn()
    else
    {
        anstream::print!("{text}");
        return;
    };

    if let Some(mut stdin) = child.stdin.take()
    {
        let _ = stdin.write_all(text.as_bytes());
    }
    let _ = child.wait();
}

// The palette: git-compatible colors shared by every renderer.
pub(crate) const STAGED: Style = green();
pub(crate) const ACTIVE: Style = green();
pub(crate) const CHANGED: Style =
    Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red)));
pub(crate) const REPO_LIST: Style =
    Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightBlack)));
pub(crate) const SKIPPED: Style =
    Style::new().effects(anstyle::Effects::DIMMED);

const fn green() -> Style
{
    Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green)))
}

/// Wraps text in a style's escape codes.
pub(crate) fn paint(style: Style, text: &str) -> String
{
    format!("{}{text}{}", style.render(), style.render_reset())
}

/// How the repo-list suffix presents its group: truncated to a few names
/// when identity is secondary, or every name when identity is the point
/// (failures).
#[derive(Clone, Copy)]
pub(crate) enum SuffixPolicy
{
    Truncated,
    Full
}

const SUFFIX_NAME_LIMIT: usize = 3;

/// The repo-list suffix: the parenthesised list of child repo names appended,
/// with its leading space, to a line that does not apply to every child repo
/// in scope. Empty when the group covers all `total` repos — no suffix means
/// "everyone".
pub(crate) fn repo_list_suffix(
    repos: &[&str],
    total: usize,
    policy: SuffixPolicy
) -> String
{
    if repos.len() == total
    {
        return String::new();
    }

    let names = match policy
    {
        SuffixPolicy::Truncated if repos.len() > SUFFIX_NAME_LIMIT => format!(
            "{}, +{}",
            repos[..SUFFIX_NAME_LIMIT].join(", "),
            repos.len() - SUFFIX_NAME_LIMIT
        ),
        SuffixPolicy::Truncated | SuffixPolicy::Full => repos.join(", ")
    };

    format!(" {}", paint(REPO_LIST, &format!("({names})")))
}

#[cfg(test)]
mod pager_tests
{
    use super::*;
    use std::fs;

    fn paged(stdout: &str, argv: Vec<String>) -> Rendered
    {
        Rendered {
            stdout: stdout.to_owned(),
            stderr: String::new(),
            pager: Some(argv)
        }
    }

    fn sh(script: String) -> Vec<String>
    {
        vec!["sh".to_owned(), "-c".to_owned(), script]
    }

    #[test]
    fn pager_receives_stdout_verbatim()
    {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("recorded");
        let f = file.display().to_string().replace('\\', "/");
        let stdout = "diff --git a/x b/x\n\x1b[32m+new\x1b[0m\n";

        let rendered = paged(stdout, sh(format!("cat > '{f}'")));
        emit(Ok(rendered)).unwrap();

        assert_eq!(fs::read(&file).unwrap(), stdout.as_bytes());
    }

    #[test]
    fn empty_stdout_does_not_launch_the_pager()
    {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("recorded");

        let rendered = paged("", sh(format!("touch {}", file.display())));
        emit(Ok(rendered)).unwrap();

        assert!(!file.exists());
    }

    #[test]
    fn emit_waits_for_the_pager_to_exit()
    {
        // The marker is appended after `cat` finishes, so seeing it as
        // soon as emit returns proves emit waited for the pager — and
        // stderr, printed after that wait, lands after the pager exits
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("recorded");
        let f = file.display().to_string().replace('\\', "/");

        let rendered =
            paged("body\n", sh(format!("cat > '{f}'; echo EXITED >> '{f}'")));
        emit(Ok(rendered)).unwrap();

        assert_eq!(fs::read_to_string(&file).unwrap(), "body\nEXITED\n");
    }

    #[test]
    #[cfg(unix)]
    fn dirname_of_argv0_is_prepended_to_the_pager_path()
    {
        // A recording script invoked by absolute path echoes the PATH it
        // sees; its own directory leading that PATH proves the prepend
        use std::os::unix::fs::PermissionsExt;

        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("recorded");
        let script = tmp.path().join("fakepager");
        fs::write(
            &script,
            format!("#!/bin/sh\necho \"$PATH\" > {}\n", file.display())
        )
        .unwrap();
        fs::set_permissions(&script, fs::Permissions::from_mode(0o755))
            .unwrap();

        let rendered =
            paged("diff body\n", vec![script.to_str().unwrap().to_owned()]);
        emit(Ok(rendered)).unwrap();

        let seen = fs::read_to_string(&file).unwrap();
        let expected = format!(
            "{}:{}",
            tmp.path().display(),
            std::env::var("PATH").unwrap()
        );
        assert_eq!(seen.trim_end(), expected);
    }

    #[test]
    fn unspawnable_argv0_prints_directly_without_failing()
    {
        // Silent designed degradation: emit succeeds and the failure
        // message still propagates unchanged, so the exit code is what it
        // would have been without a pager
        let argv = vec!["/nonexistent/definitely-missing-pager".to_owned()];

        emit(Ok(paged("diff body\n", argv.clone()))).unwrap();

        let error = emit(Err(fail(
            paged("diff body\n", argv),
            "one failed".to_owned()
        )))
        .unwrap_err();
        assert_eq!(error.to_string(), "one failed");
    }

    #[test]
    fn a_pager_that_dies_loses_the_diff_without_failing_the_command()
    {
        // The pager exits without reading; the broken pipe and nonzero
        // exit are swallowed and nothing is re-printed
        let rendered = paged(&"x".repeat(1 << 20), sh("exit 7".to_owned()));

        emit(Ok(rendered)).unwrap();
    }

    #[test]
    fn less_and_lv_get_git_defaults_when_unset()
    {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("recorded");
        let f = file.display().to_string().replace('\\', "/");
        let expected = format!(
            "{} {}\n",
            std::env::var("LESS").unwrap_or_else(|_| "FRX".to_owned()),
            std::env::var("LV").unwrap_or_else(|_| "-c".to_owned())
        );

        let rendered = paged(
            "body\n",
            sh(format!(r#"cat >/dev/null; echo "$LESS $LV" > '{f}'"#))
        );
        emit(Ok(rendered)).unwrap();

        assert_eq!(fs::read_to_string(&file).unwrap(), expected);
    }

    #[test]
    fn failure_output_is_paged_before_the_message_propagates()
    {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("recorded");
        let f = file.display().to_string().replace('\\', "/");

        let mut rendered =
            paged("surviving diff\n", sh(format!("cat > '{f}'")));
        rendered.stderr = String::new();
        let error = emit(Err(fail(rendered, "one repo failed".to_owned())))
            .unwrap_err();

        assert_eq!(error.to_string(), "one repo failed");
        assert_eq!(fs::read_to_string(&file).unwrap(), "surviving diff\n");
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn repo_list_suffix_renders_gray_names_with_leading_space()
    {
        let suffix = repo_list_suffix(
            &["backend", "frontend"],
            3,
            SuffixPolicy::Truncated
        );

        assert_eq!(suffix, " \x1b[90m(backend, frontend)\x1b[0m");
    }

    #[test]
    fn repo_list_suffix_is_omitted_when_group_covers_scope()
    {
        for policy in [SuffixPolicy::Truncated, SuffixPolicy::Full]
        {
            assert_eq!(repo_list_suffix(&["a", "b"], 2, policy), "");
        }
    }

    #[test]
    fn truncated_suffix_keeps_all_names_at_the_limit()
    {
        let suffix =
            repo_list_suffix(&["a", "b", "c"], 4, SuffixPolicy::Truncated);

        assert_eq!(suffix, " \x1b[90m(a, b, c)\x1b[0m");
    }

    #[test]
    fn truncated_suffix_replaces_names_past_the_limit_with_a_count()
    {
        let suffix = repo_list_suffix(
            &["a", "b", "c", "d", "e"],
            6,
            SuffixPolicy::Truncated
        );

        assert_eq!(suffix, " \x1b[90m(a, b, c, +2)\x1b[0m");
    }

    #[test]
    fn full_suffix_never_truncates()
    {
        let suffix =
            repo_list_suffix(&["a", "b", "c", "d", "e"], 6, SuffixPolicy::Full);

        assert_eq!(suffix, " \x1b[90m(a, b, c, d, e)\x1b[0m");
    }

    #[test]
    fn single_name_suffix_renders_plainly_under_both_policies()
    {
        for policy in [SuffixPolicy::Truncated, SuffixPolicy::Full]
        {
            assert_eq!(
                repo_list_suffix(&["tools"], 2, policy),
                " \x1b[90m(tools)\x1b[0m"
            );
        }
    }
}
