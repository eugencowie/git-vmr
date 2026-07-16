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
    pub stderr: String
}

impl From<String> for Rendered
{
    fn from(stdout: String) -> Self
    {
        Self { stdout, stderr: String::new() }
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
    anstream::print!("{}", rendered.stdout);
    anstream::eprint!("{}", rendered.stderr);
}

// The palette: git-compatible colors shared by every renderer.
pub(crate) const STAGED: Style = green();
pub(crate) const ACTIVE: Style = green();
pub(crate) const CHANGED: Style =
    Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red)));
pub(crate) const REPO_LIST: Style =
    Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightBlack)));

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
