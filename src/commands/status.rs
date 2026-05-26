use crate::git::{self, FileChange, FileEntry, Head, RepoStatus};
use crate::vmr::{Repo, Vmr};
use anstyle::{AnsiColor, Style};
use anyhow::Result;
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::path::Path;

struct RenderContext<'a>
{
    repos: &'a [(&'a Repo, &'a RepoStatus)],
    bin_name: &'a str,
    working_dir: &'a Path
}

#[derive(Clone, Copy)]
struct StatusStyle(Style);

impl StatusStyle
{
    fn render_text(self, text: &str) -> String
    {
        // Wrap text in style escape codes
        format!("{}{text}{}", self.0.render(), self.0.render_reset())
    }
}

struct StatusStyles
{
    staged: StatusStyle,
    changed: StatusStyle,
    repo_list: StatusStyle
}

impl StatusStyles
{
    fn new() -> Self
    {
        // Configure git-compatible status colors
        Self {
            staged: StatusStyle(
                Style::new().fg_color(Some(AnsiColor::Green.into()))
            ),
            changed: StatusStyle(
                Style::new().fg_color(Some(AnsiColor::Red.into()))
            ),
            repo_list: StatusStyle(
                Style::new().fg_color(Some(AnsiColor::BrightBlack.into()))
            )
        }
    }
}

pub fn status(bin_name: &str, working_dir: &Path) -> Result<()>
{
    // Find virtual monorepo
    let vmr = Vmr::find(working_dir)?;

    // Get list of repositories
    let repos = vmr.repos()?;

    // Collect status information from repositories
    let mut statuses = repos
        .par_iter()
        .filter_map(|repo| git::status(repo).transpose())
        .collect::<Result<Vec<_>>>()?;

    // Keep status order deterministic
    statuses.sort_by(|(a, _), (b, _)| a.name.cmp(&b.name));

    // Render status output
    anstream::print!("{}", render_status(&statuses, bin_name, working_dir));

    Ok(())
}

fn render_status(
    statuses: &[(Repo, RepoStatus)],
    bin_name: &str,
    working_dir: &Path
) -> String
{
    let mut output = String::new();
    let styles = StatusStyles::new();

    let mut branch_groups: BTreeMap<(&str, bool), Vec<(&Repo, &RepoStatus)>> =
        BTreeMap::new();
    let mut detached = Vec::new();

    // Group statuses by branch
    for (repo, status) in statuses
    {
        match &status.head
        {
            Head::Branch(branch) => branch_groups
                .entry((branch, status.initial))
                .or_default()
                .push((repo, status)),
            Head::Detached(hash) => detached.push((repo, hash.as_str(), status))
        }
    }

    let largest_committed_branch = largest_committed_branch(&branch_groups);

    // Render branch groups
    for ((branch, initial), repos) in branch_groups
    {
        let repo_names = repos
            .iter()
            .map(|(repo, _)| repo.name.as_str())
            .collect::<Vec<_>>();
        if !initial && largest_committed_branch == Some(branch)
        {
            output.push_str(&format!("On branch {}\n", branch));
        }
        else
        {
            output.push_str(&format!(
                "On branch {} {}\n",
                branch,
                styles
                    .repo_list
                    .render_text(&format!("({})", repo_names.join(", ")))
            ));
        }

        render_group(&mut output, &repos, bin_name, working_dir, &styles);
    }

    // Render detached repositories
    for (repo, hash, status) in detached
    {
        output.push_str(&format!(
            "HEAD detached at {} {}\n",
            hash,
            styles.repo_list.render_text(&format!("({})", repo.name))
        ));
        render_group(
            &mut output,
            &[(repo, status)],
            bin_name,
            working_dir,
            &styles
        );
    }

    if output.ends_with("\n\n")
    {
        output.pop();
    }

    output
}

fn largest_committed_branch<'a>(
    branch_groups: &BTreeMap<(&'a str, bool), Vec<(&Repo, &RepoStatus)>>
) -> Option<&'a str>
{
    let mut largest = None;
    let mut largest_size = 0;
    let mut tie = false;

    for ((branch, initial), repos) in branch_groups
    {
        if *initial
        {
            continue;
        }

        let size = repos.len();
        if size > largest_size
        {
            largest = Some(*branch);
            largest_size = size;
            tie = false;
        }
        else if size == largest_size
        {
            tie = true;
        }
    }

    if tie { None } else { largest }
}

fn render_group(
    output: &mut String,
    repos: &[(&Repo, &RepoStatus)],
    bin_name: &str,
    working_dir: &Path,
    styles: &StatusStyles
)
{
    let context = RenderContext { repos, bin_name, working_dir };

    // Render initial commit notice
    let has_initial = repos.iter().any(|(_, status)| status.initial);
    if has_initial
    {
        output.push_str("\nNo commits yet\n");
    }

    // Render change sections
    let has_staged = render_paths(
        output,
        "Changes to be committed:",
        &[format!(
            "  (use \"{} restore --staged <file>...\" to unstage)",
            context.bin_name
        )],
        &context,
        |status| &status.staged_changes,
        styles.staged
    );
    let has_unstaged = render_paths(
        output,
        "Changes not staged for commit:",
        &[
            format!(
                "  (use \"{} add <file>...\" to update what will be committed)",
                context.bin_name
            ),
            format!(
                "  (use \"{} restore <file>...\" to discard changes in working directory)",
                context.bin_name
            )
        ],
        &context,
        |status| &status.unstaged_changes,
        styles.changed
    );
    let has_untracked = render_paths(
        output,
        "Untracked files:",
        &[format!(
            "  (use \"{} add <file>...\" to include in what will be committed)",
            context.bin_name
        )],
        &context,
        |status| &status.untracked_files,
        styles.changed
    );

    // Render clean summary
    if has_staged || has_unstaged || has_untracked || has_initial
    {
        output.push('\n');
    }
    else if repos.iter().all(|(_, status)| !status.initial)
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
    style: StatusStyle
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
            style.render_text(file_change_label(change)),
            " ".repeat(12 - file_change_label(change).len()),
            style.render_text(&path)
        ));
    }

    true
}

#[cfg(test)]
mod tests
{
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;

    const BIN_NAME: &str = "git-vmr";

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
        let output = render_status(&statuses, BIN_NAME, tmp.path());

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
        let output = render_status(&statuses, BIN_NAME, tmp.path());

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
        let output = render_status(&statuses, BIN_NAME, tmp.path());

        // Assert
        assert!(output.contains("main"));
        assert!(output.contains("\u{1b}[90m(backend)\u{1b}[0m"));
        assert!(output.contains("feature/auth"));
        assert!(output.contains("\u{1b}[90m(frontend)\u{1b}[0m"));
    }

    #[test]
    fn renders_largest_branch_group_without_repo_names()
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
        let output = render_status(&statuses, BIN_NAME, tmp.path());

        // Assert
        assert!(output.contains("On branch develop\n"));
        assert!(!output.contains(
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
    fn renders_detached_head()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let mut status = repo_status("main");
        status.head = Head::Detached("a1b2c3d".to_owned());
        let statuses = vec![(repo(tmp.path(), "tools"), status)];

        // Act
        let output = render_status(&statuses, BIN_NAME, tmp.path());

        // Assert
        assert!(output.contains("HEAD detached at "));
        assert!(output.contains("a1b2c3d"));
        assert!(output.contains("\u{1b}[90m(tools)\u{1b}[0m"));
        assert!(!output.contains("\u{1b}[33m"));
    }

    #[test]
    fn renders_no_commits()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let mut status = repo_status("master");
        status.initial = true;
        let statuses = vec![(repo(tmp.path(), "new-repo"), status)];

        // Act
        let output = render_status(&statuses, BIN_NAME, tmp.path());

        // Assert
        assert!(output.contains("master"));
        assert!(output.contains("\u{1b}[90m(new-repo)\u{1b}[0m"));
        assert!(output.contains("\nNo commits yet\n"));
    }

    #[test]
    fn renders_initial_repos_separately_from_committed_repos_on_same_branch()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let mut initial = repo_status("master");
        initial.initial = true;
        let statuses = vec![
            (repo(tmp.path(), "committed"), repo_status("master")),
            (repo(tmp.path(), "new-repo"), initial),
        ];

        // Act
        let output = render_status(&statuses, BIN_NAME, tmp.path());

        // Assert
        assert!(output.contains(
            "On branch master\n\nnothing to commit, working tree clean\n\n"
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
        let output = render_status(&statuses, BIN_NAME, tmp.path());

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
        let mut status = repo_status("main");
        status
            .unstaged_changes
            .push(file_entry("src/main.rs", FileChange::Modified));
        let statuses = vec![(repo(tmp.path(), "backend"), status)];

        // Act
        let output = render_status(&statuses, BIN_NAME, &working_dir);

        // Assert
        assert!(output.contains("../backend/src/main.rs"));
        assert!(output.contains("\u{1b}[31m"));
    }

    #[test]
    fn renders_paths_with_git_style_separators()
    {
        // Arrange
        let path = Path::new("frontend\\docs");

        // Act
        let output = git::git_style_path(path);

        // Assert
        assert_eq!(output, "frontend/docs");
    }

    #[test]
    fn renders_hints_with_invoked_command_name()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let mut status = repo_status("main");
        status
            .staged_changes
            .push(file_entry("staged.rs", FileChange::Modified));
        status
            .unstaged_changes
            .push(file_entry("unstaged.rs", FileChange::Modified));
        status
            .untracked_files
            .push(file_entry("untracked.rs", FileChange::NewFile));
        let statuses = vec![(repo(tmp.path(), "backend"), status)];

        // Act
        let output = render_status(&statuses, "vv", tmp.path());

        // Assert
        assert!(output.contains(r#"(use "vv restore --staged <file>..."#));
        assert!(output.contains(r#"(use "vv add <file>..."#));
        assert!(output.contains(r#"(use "vv restore <file>..."#));
        assert!(!output.contains("git vmr"));
    }

    #[test]
    fn status_succeeds_with_git_repos_and_non_git_dirs()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir(tmp.path().join(".gitvmr")).unwrap();
        fs::create_dir(tmp.path().join("docs")).unwrap();
        init_git_repo(&tmp.path().join("backend"));

        // Act
        let result = status(BIN_NAME, tmp.path());

        // Assert
        assert!(result.is_ok());
    }

    fn repo(root: &Path, name: &str) -> Repo
    {
        Repo { name: name.to_owned(), path: root.join(name) }
    }

    fn init_git_repo(path: &Path)
    {
        fs::create_dir(path).unwrap();

        Command::new("git").arg("init").current_dir(path).output().unwrap();
    }

    fn repo_status(branch: &str) -> RepoStatus
    {
        RepoStatus {
            head: Head::Branch(branch.to_owned()),
            initial: false,
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
