use crate::cli::CliContext;
use crate::render::{
    CHANGED, Rendered, STAGED, SuffixPolicy, paint, repo_list_suffix
};
use crate::workspace::Workspace;
use anyhow::Result;

pub fn run(workspace: &Workspace, context: &CliContext) -> Result<Rendered>
{
    let display_name = &context.display_name;
    let working_dir = &context.working_dir;

    // Collect status information from child repositories, in repo order
    let statuses = workspace
        .map(|git, repo| git.status(repo))?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    // Render status output
    Ok(render(&statuses, display_name, working_dir).into())
}

use crate::git::{self, Git, Head};
use crate::vmr::Repo;
use anyhow::Context;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FileChange
{
    NewFile,
    Modified,
    Deleted,
    Renamed,
    TypeChange,
    Copied
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FileEntry
{
    path: PathBuf,
    change: FileChange
}

#[derive(Clone, PartialEq, Eq)]
struct RepoStatus
{
    head: Head,
    staged_changes: Vec<FileEntry>,
    unstaged_changes: Vec<FileEntry>,
    untracked_files: Vec<FileEntry>
}

impl Git
{
    fn status(&self, repo: &Repo) -> Result<Option<(Repo, RepoStatus)>>
    {
        let status_output = self
            .stdout(&repo.path, [
                "status",
                "--porcelain=v1",
                "-z",
                "--branch",
                "--no-ahead-behind"
            ])
            .with_context(|| {
                format!(
                    "fatal: failed to read git status for '{}'",
                    repo.path.display()
                )
            })?;
        let mut records = status_output
            .split(|byte| *byte == 0)
            .filter(|record| !record.is_empty());

        let branch_header = records
            .next()
            .context("fatal: git status did not return a branch header")?;
        let head = self.head_from_status_header(
            &repo.path,
            "failed to read git status",
            branch_header
        )?;

        let (staged_changes, unstaged_changes, untracked_files) =
            parse_file_records(records);

        Ok(Some((repo.clone(), RepoStatus {
            head,
            staged_changes,
            unstaged_changes,
            untracked_files
        })))
    }
}

type FileEntries = (Vec<FileEntry>, Vec<FileEntry>, Vec<FileEntry>);

/// Parses porcelain v1 `-z` file records into staged, unstaged and untracked
/// entries. Pure: bytes in, entries out.
fn parse_file_records<'a>(
    mut records: impl Iterator<Item = &'a [u8]>
) -> FileEntries
{
    let mut staged_changes = Vec::new();
    let mut unstaged_changes = Vec::new();
    let mut untracked_files = Vec::new();

    while let Some(record) = records.next()
    {
        let record = String::from_utf8_lossy(record);
        if record.len() < 4
        {
            continue;
        }

        let mut chars = record.chars();
        let index_status = chars.next().unwrap_or(' ');
        let worktree_status = chars.next().unwrap_or(' ');
        let path = PathBuf::from(record[3..].to_owned());

        if matches!(index_status, 'R' | 'C')
            || matches!(worktree_status, 'R' | 'C')
        {
            let _old_path = records.next();
        }

        if index_status == '?' && worktree_status == '?'
        {
            untracked_files
                .push(FileEntry { path, change: FileChange::NewFile });
            continue;
        }

        if index_status == ' ' && worktree_status == 'A'
        {
            staged_changes
                .push(FileEntry { path, change: FileChange::NewFile });
            continue;
        }

        if let Some(change) = parse_status_change(index_status)
        {
            staged_changes.push(FileEntry { path: path.clone(), change });
        }
        if let Some(change) = parse_status_change(worktree_status)
        {
            unstaged_changes.push(FileEntry { path, change });
        }
    }

    (staged_changes, unstaged_changes, untracked_files)
}

fn parse_status_change(status: char) -> Option<FileChange>
{
    match status
    {
        'A' => Some(FileChange::NewFile),
        'M' => Some(FileChange::Modified),
        'D' => Some(FileChange::Deleted),
        'R' => Some(FileChange::Renamed),
        'T' => Some(FileChange::TypeChange),
        'C' => Some(FileChange::Copied),
        'U' => Some(FileChange::Modified),
        _ => None
    }
}

use anstyle::Style;
use std::collections::BTreeMap;
use std::path::Path;

struct RenderContext<'a>
{
    repos: &'a [(&'a Repo, &'a RepoStatus)],
    display_name: &'a str,
    working_dir: &'a Path
}

/// Renders working tree status grouped by branch.
fn render(
    statuses: &[(Repo, RepoStatus)],
    display_name: &str,
    working_dir: &Path
) -> String
{
    let mut output = String::new();

    let mut branch_groups: BTreeMap<(&str, bool), Vec<(&Repo, &RepoStatus)>> =
        BTreeMap::new();
    let mut detached = Vec::new();

    // Group statuses by branch, keeping unborn branches apart from committed
    // ones that share the name
    for (repo, status) in statuses
    {
        match &status.head
        {
            Head::Branch(branch) => branch_groups
                .entry((branch, false))
                .or_default()
                .push((repo, status)),
            Head::Unborn(branch) => branch_groups
                .entry((branch, true))
                .or_default()
                .push((repo, status)),
            Head::Detached(hash) => detached.push((repo, hash.as_str(), status))
        }
    }

    // Render branch groups
    for ((branch, _), repos) in branch_groups
    {
        let repo_names = repos
            .iter()
            .map(|(repo, _)| repo.name.as_str())
            .collect::<Vec<_>>();
        output.push_str(&format!(
            "On branch {}{}\n",
            branch,
            repo_list_suffix(
                &repo_names,
                statuses.len(),
                SuffixPolicy::Truncated
            )
        ));

        render_group(&mut output, &repos, display_name, working_dir);
    }

    // Render detached repositories
    for (repo, hash, status) in detached
    {
        output.push_str(&format!(
            "HEAD detached at {}{}\n",
            hash,
            repo_list_suffix(
                &[repo.name.as_str()],
                statuses.len(),
                SuffixPolicy::Truncated
            )
        ));
        render_group(&mut output, &[(repo, status)], display_name, working_dir);
    }

    if output.ends_with("\n\n")
    {
        output.pop();
    }

    output
}

fn render_group(
    output: &mut String,
    repos: &[(&Repo, &RepoStatus)],
    display_name: &str,
    working_dir: &Path
)
{
    let context = RenderContext { repos, display_name, working_dir };

    // Render initial commit notice
    let has_unborn =
        repos.iter().any(|(_, status)| matches!(status.head, Head::Unborn(_)));
    if has_unborn
    {
        output.push_str("\nNo commits yet\n");
    }

    // Render change sections
    let has_staged = render_paths(
        output,
        "Changes to be committed:",
        &[format!(
            "  (use \"{} restore --staged <file>...\" to unstage)",
            context.display_name
        )],
        &context,
        |status| &status.staged_changes,
        STAGED
    );
    let has_unstaged = render_paths(
        output,
        "Changes not staged for commit:",
        &[
            format!(
                "  (use \"{} add <file>...\" to update what will be committed)",
                context.display_name
            ),
            format!(
                "  (use \"{} restore <file>...\" to discard changes in working directory)",
                context.display_name
            )
        ],
        &context,
        |status| &status.unstaged_changes,
        CHANGED
    );
    let has_untracked = render_paths(
        output,
        "Untracked files:",
        &[format!(
            "  (use \"{} add <file>...\" to include in what will be committed)",
            context.display_name
        )],
        &context,
        |status| &status.untracked_files,
        CHANGED
    );

    // Render clean summary
    if has_staged || has_unstaged || has_untracked || has_unborn
    {
        output.push('\n');
    }
    else
    {
        output.push_str("\nnothing to commit, working tree clean\n\n");
    }
}

fn file_change_label(change: FileChange) -> &'static str
{
    // Convert change kind to status label
    match change
    {
        FileChange::NewFile => "new file:",
        FileChange::Modified => "modified:",
        FileChange::Deleted => "deleted:",
        FileChange::Renamed => "renamed:",
        FileChange::TypeChange => "typechange:",
        FileChange::Copied => "copied:"
    }
}

fn render_paths(
    output: &mut String,
    heading: &str,
    hints: &[String],
    context: &RenderContext,
    paths: fn(&RepoStatus) -> &Vec<FileEntry>,
    style: Style
) -> bool
{
    // Collect paths for this group
    let mut rendered: Vec<(String, FileChange)> = Vec::new();
    for (repo, status) in context.repos
    {
        let repo_prefix = pathdiff::diff_paths(&repo.path, context.working_dir)
            .unwrap_or(repo.path.clone());
        for entry in paths(status)
        {
            let rel = repo_prefix.join(&entry.path);
            rendered.push((git::git_style_path(&rel), entry.change));
        }
    }

    // Sort paths by rendered name
    rendered.sort_by(|a, b| a.0.cmp(&b.0));

    // Skip empty sections
    if rendered.is_empty()
    {
        return false;
    }

    // Render section heading and hints
    output.push('\n');
    output.push_str(heading);
    output.push('\n');
    for hint in hints
    {
        output.push_str(hint);
        output.push('\n');
    }

    // Render changed paths
    for (path, change) in rendered
    {
        output.push_str(&format!(
            "\t{}{}{}\n",
            paint(style, file_change_label(change)),
            " ".repeat(12 - file_change_label(change).len()),
            paint(style, &path)
        ));
    }

    true
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::git::{Head, ScriptedFake};
    use crate::vmr::Repo;

    const STATUS_ARGS: [&str; 5] =
        ["status", "--porcelain=v1", "-z", "--branch", "--no-ahead-behind"];

    fn repo() -> Repo
    {
        Repo { name: "backend".to_owned(), path: PathBuf::from("/vmr/backend") }
    }

    #[test]
    fn status_parses_staged_unstaged_untracked_and_rename_records()
    {
        // Arrange
        let git = Git::with(ScriptedFake::new().on(
            STATUS_ARGS,
            0,
            "## main...origin/main\0M  staged.rs\0 D removed.rs\0?? new.rs\0R  renamed.rs\0old.rs\0",
            ""
        ));

        // Act
        let (_, status) = git.status(&repo()).unwrap().unwrap();

        // Assert
        assert!(matches!(&status.head, Head::Branch(name) if name == "main"));
        assert_eq!(status.staged_changes, vec![
            FileEntry {
                path: PathBuf::from("staged.rs"),
                change: FileChange::Modified
            },
            FileEntry {
                path: PathBuf::from("renamed.rs"),
                change: FileChange::Renamed
            }
        ]);
        assert_eq!(status.unstaged_changes, vec![FileEntry {
            path: PathBuf::from("removed.rs"),
            change: FileChange::Deleted
        }]);
        assert_eq!(status.untracked_files, vec![FileEntry {
            path: PathBuf::from("new.rs"),
            change: FileChange::NewFile
        }]);
    }

    #[test]
    fn status_resolves_detached_head_with_a_second_invocation()
    {
        // Arrange
        let git = Git::with(
            ScriptedFake::new()
                .on(STATUS_ARGS, 0, "## HEAD (no branch)\0", "")
                .on(["rev-parse", "--short=8", "HEAD"], 0, "abc12345\n", "")
        );

        // Act
        let (_, status) = git.status(&repo()).unwrap().unwrap();

        // Assert
        assert!(
            matches!(&status.head, Head::Detached(hash) if hash == "abc12345")
        );
    }

    #[test]
    fn status_reports_unborn_branch_before_first_commit()
    {
        // Arrange
        let git = Git::with(ScriptedFake::new().on(
            STATUS_ARGS,
            0,
            "## No commits yet on main\0",
            ""
        ));

        // Act
        let (_, status) = git.status(&repo()).unwrap().unwrap();

        // Assert
        assert!(matches!(&status.head, Head::Unborn(name) if name == "main"));
    }

    #[test]
    fn parse_file_records_treats_index_space_worktree_a_as_staged()
    {
        // Act
        let (staged, unstaged, untracked) =
            parse_file_records([b" A intent.rs" as &[u8]].into_iter());

        // Assert
        assert_eq!(staged, vec![FileEntry {
            path: PathBuf::from("intent.rs"),
            change: FileChange::NewFile
        }]);
        assert!(unstaged.is_empty());
        assert!(untracked.is_empty());
    }
}

#[cfg(test)]
mod render_tests
{
    use super::*;
    use std::path::PathBuf;

    const DISPLAY_NAME: &str = "git vmr";

    #[test]
    fn renders_single_branch_without_repo_list()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let statuses = vec![
            (repo(tmp.path(), "backend"), repo_status("main")),
            (repo(tmp.path(), "frontend"), repo_status("main")),
        ];

        // Act
        let output = render(&statuses, DISPLAY_NAME, tmp.path());

        // Assert
        assert!(output.contains("On branch "));
        assert!(output.contains("main"));
        assert!(!output.contains("(backend"));
        assert!(!output.contains("\u{1b}[32m"));
    }

    #[test]
    fn renders_clean_summary_for_clean_branch_group()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let statuses = vec![(repo(tmp.path(), "backend"), repo_status("main"))];

        // Act
        let output = render(&statuses, DISPLAY_NAME, tmp.path());

        // Assert
        assert!(output.contains("On branch main"));
        assert!(output.contains(
            "On branch main\n\nnothing to commit, working tree clean\n"
        ));
    }

    #[test]
    fn renders_diverged_branches_with_repo_names()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let statuses = vec![
            (repo(tmp.path(), "backend"), repo_status("main")),
            (repo(tmp.path(), "frontend"), repo_status("feature/auth")),
        ];

        // Act
        let output = render(&statuses, DISPLAY_NAME, tmp.path());

        // Assert
        assert!(output.contains("main"));
        assert!(output.contains("\u{1b}[90m(backend)\u{1b}[0m"));
        assert!(output.contains("feature/auth"));
        assert!(output.contains("\u{1b}[90m(frontend)\u{1b}[0m"));
    }

    #[test]
    fn renders_every_partial_branch_group_with_repo_names()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let statuses = vec![
            (repo(tmp.path(), "backend"), repo_status("develop")),
            (repo(tmp.path(), "common"), repo_status("new-feature")),
            (repo(tmp.path(), "docs"), repo_status("develop")),
            (repo(tmp.path(), "frontend"), repo_status("develop")),
            (repo(tmp.path(), "shared"), repo_status("new-feature")),
            (repo(tmp.path(), "tools"), repo_status("bug-fix")),
        ];

        // Act
        let output = render(&statuses, DISPLAY_NAME, tmp.path());

        // Assert
        assert!(output.contains(
            "On branch develop \u{1b}[90m(backend, docs, frontend)\u{1b}[0m"
        ));
        assert!(output.contains(
            "On branch new-feature \u{1b}[90m(common, shared)\u{1b}[0m"
        ));
        assert!(
            output.contains("On branch bug-fix \u{1b}[90m(tools)\u{1b}[0m")
        );
    }

    #[test]
    fn truncates_long_branch_group_repo_names()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let statuses = vec![
            (repo(tmp.path(), "a"), repo_status("develop")),
            (repo(tmp.path(), "b"), repo_status("develop")),
            (repo(tmp.path(), "c"), repo_status("develop")),
            (repo(tmp.path(), "d"), repo_status("develop")),
            (repo(tmp.path(), "e"), repo_status("develop")),
            (repo(tmp.path(), "f"), repo_status("bug-fix")),
        ];

        // Act
        let output = render(&statuses, DISPLAY_NAME, tmp.path());

        // Assert
        assert!(
            output
                .contains("On branch develop \u{1b}[90m(a, b, c, +2)\u{1b}[0m")
        );
    }

    #[test]
    fn renders_detached_head()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let mut repo_status = repo_status("main");
        repo_status.head = Head::Detached("a1b2c3d".to_owned());
        let statuses = vec![(repo(tmp.path(), "tools"), repo_status)];

        // Act
        let output = render(&statuses, DISPLAY_NAME, tmp.path());

        // Assert
        assert!(output.contains("HEAD detached at a1b2c3d\n"));
        assert!(!output.contains("(tools)"));
        assert!(!output.contains("\u{1b}[33m"));
    }

    #[test]
    fn renders_detached_head_with_repo_names_when_not_all_repos_detached()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let mut detached_status = repo_status("main");
        detached_status.head = Head::Detached("a1b2c3d".to_owned());
        let statuses = vec![
            (repo(tmp.path(), "backend"), repo_status("main")),
            (repo(tmp.path(), "tools"), detached_status),
        ];

        // Act
        let output = render(&statuses, DISPLAY_NAME, tmp.path());

        // Assert
        assert!(
            output.contains(
                "HEAD detached at a1b2c3d \u{1b}[90m(tools)\u{1b}[0m\n"
            )
        );
    }

    #[test]
    fn renders_no_commits()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let mut repo_status = repo_status("master");
        repo_status.head = Head::Unborn("master".to_owned());
        let statuses = vec![(repo(tmp.path(), "new-repo"), repo_status)];

        // Act
        let output = render(&statuses, DISPLAY_NAME, tmp.path());

        // Assert
        assert!(output.contains("On branch master\n"));
        assert!(!output.contains("(new-repo)"));
        assert!(output.contains("\nNo commits yet\n"));
    }

    #[test]
    fn renders_unborn_repos_separately_from_committed_repos_on_same_branch()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let mut unborn = repo_status("master");
        unborn.head = Head::Unborn("master".to_owned());
        let statuses = vec![
            (repo(tmp.path(), "committed"), repo_status("master")),
            (repo(tmp.path(), "new-repo"), unborn),
        ];

        // Act
        let output = render(&statuses, DISPLAY_NAME, tmp.path());

        // Assert
        assert!(output.contains(
            "On branch master \u{1b}[90m(committed)\u{1b}[0m\n\n\
             nothing to commit, working tree clean\n\n"
        ));
        assert!(output.contains(
            "On branch master \u{1b}[90m(new-repo)\u{1b}[0m\n\nNo commits yet\n"
        ));
        assert!(!output.contains("On branch master (committed, new-repo)"));
    }

    #[test]
    fn renders_blank_lines_between_clean_branch_groups()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let statuses = vec![
            (
                repo(tmp.path(), "backend"),
                repo_status("create-project-structure")
            ),
            (repo(tmp.path(), "frontend"), repo_status("develop")),
        ];

        // Act
        let output = render(&statuses, DISPLAY_NAME, tmp.path());

        // Assert
        assert!(output.contains(
            "On branch create-project-structure \u{1b}[90m(backend)\u{1b}[0m\n\n\
             nothing to commit, working tree clean\n\n\
             On branch develop \u{1b}[90m(frontend)\u{1b}[0m\n\n\
             nothing to commit, working tree clean\n"
        ));
    }

    #[test]
    fn renders_paths_relative_to_working_dir()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let working_dir = tmp.path().join("frontend");
        let mut repo_status = repo_status("main");
        repo_status
            .unstaged_changes
            .push(file_entry("src/main.rs", FileChange::Modified));
        let statuses = vec![(repo(tmp.path(), "backend"), repo_status)];

        // Act
        let output = render(&statuses, DISPLAY_NAME, &working_dir);

        // Assert
        assert!(output.contains("../backend/src/main.rs"));
        assert!(output.contains("\u{1b}[31m"));
    }

    #[test]
    fn renders_hints_with_invoked_command_name()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let mut repo_status = repo_status("main");
        repo_status
            .staged_changes
            .push(file_entry("staged.rs", FileChange::Modified));
        repo_status
            .unstaged_changes
            .push(file_entry("unstaged.rs", FileChange::Modified));
        repo_status
            .untracked_files
            .push(file_entry("untracked.rs", FileChange::NewFile));
        let statuses = vec![(repo(tmp.path(), "backend"), repo_status)];

        // Act
        let output = render(&statuses, "vv", tmp.path());

        // Assert
        assert!(output.contains(r#"(use "vv restore --staged <file>..."#));
        assert!(output.contains(r#"(use "vv add <file>..."#));
        assert!(output.contains(r#"(use "vv restore <file>..."#));
        assert!(!output.contains("git vmr"));
    }

    fn repo(root: &Path, name: &str) -> Repo
    {
        Repo { name: name.to_owned(), path: root.join(name) }
    }

    fn repo_status(branch: &str) -> RepoStatus
    {
        RepoStatus {
            head: Head::Branch(branch.to_owned()),
            staged_changes: Vec::new(),
            unstaged_changes: Vec::new(),
            untracked_files: Vec::new()
        }
    }

    fn file_entry(path: &str, change: FileChange) -> FileEntry
    {
        FileEntry { path: PathBuf::from(path), change }
    }
}
