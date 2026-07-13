use crate::git::{self, FileChange, FileEntry, Head, RepoStatus};
use crate::render::{CHANGED, STAGED, SuffixPolicy, paint, repo_list_suffix};
use crate::workspace::Repo;
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
pub fn status(
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
        let output = status(&statuses, DISPLAY_NAME, tmp.path());

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
        let output = status(&statuses, DISPLAY_NAME, tmp.path());

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
        let output = status(&statuses, DISPLAY_NAME, tmp.path());

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
        let output = status(&statuses, DISPLAY_NAME, tmp.path());

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
        let output = status(&statuses, DISPLAY_NAME, tmp.path());

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
        let output = status(&statuses, DISPLAY_NAME, tmp.path());

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
        let output = status(&statuses, DISPLAY_NAME, tmp.path());

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
        let output = status(&statuses, DISPLAY_NAME, tmp.path());

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
        let output = status(&statuses, DISPLAY_NAME, tmp.path());

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
        let output = status(&statuses, DISPLAY_NAME, tmp.path());

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
        let output = status(&statuses, DISPLAY_NAME, &working_dir);

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
        let output = status(&statuses, "vv", tmp.path());

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
