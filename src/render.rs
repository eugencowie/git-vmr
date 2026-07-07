//! Rendering: turning per-repo data and outcomes into styled terminal text.
//! Rendering is pure — printing happens once, at the [`emit`] choke point.

mod branch;
mod foreach;
mod outcomes;
mod status;
mod tag;
mod worktree;

use anstyle::{AnsiColor, Color, Style};
use anyhow::Result;
pub use branch::branches;
pub use foreach::{ChildOutput, foreach};
pub use outcomes::outcomes;
pub use status::status;
use std::fmt;
pub use tag::tags;
pub use worktree::{WorktreeRootEntry, worktree_list};

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

/// The repo-list suffix: the parenthesised list of child repo names appended
/// to a message or heading that does not apply to every child repo.
pub(crate) fn repo_list_suffix(repo_names: &[&str]) -> String
{
    paint(REPO_LIST, &format!("({})", repo_names.join(", ")))
}
