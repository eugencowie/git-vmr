use crate::config::vmr;
use anstyle::{AnsiColor, Style};
use anyhow::{Context, Result, bail};
use rayon::prelude::*;
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, PartialEq, Eq)]
enum Head
{
    Branch(String),
    Detached(String)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FileChange
{
    NewFile,
    Modified,
    Deleted,
    Renamed,
    TypeChange,
    Copied
}

#[derive(Clone, PartialEq, Eq)]
struct FileEntry
{
    pub path: PathBuf,
    pub change: FileChange
}

#[derive(Clone, PartialEq, Eq)]
struct RepoStatus
{
    pub head: Head,
    pub initial: bool,
    pub staged_changes: Vec<FileEntry>,
    pub unstaged_changes: Vec<FileEntry>,
    pub untracked_files: Vec<FileEntry>
}

struct RenderContext<'a>
{
    repos: &'a [(&'a str, &'a RepoStatus)],
    working_dir: &'a Path,
    vmr_root: &'a Path
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
    changed: StatusStyle
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
            )
        }
    }
}

pub fn status(working_dir: &Path) -> Result<()>
{
    // Find VMR root
    let vmr_root = vmr::find_vmr_root(working_dir)?;

    // Collect repository statuses
    let statuses = collect_statuses(&vmr_root)?;

    // Render status output
    anstream::print!("{}", render_status(&statuses, working_dir, &vmr_root));

    Ok(())
}

fn collect_statuses(vmr_root: &Path) -> Result<Vec<(String, RepoStatus)>>
{
    // Find child repositories
    let children = fs::read_dir(vmr_root)
        .with_context(|| {
            format!("failed to read VMR root '{}'", vmr_root.display())
        })?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.file_type().map(|ty| ty.is_dir()).unwrap_or(false)
        })
        .filter(|entry| entry.file_name() != OsStr::new(".gitvmr"))
        .map(|entry| entry.path())
        .collect::<Vec<_>>();

    // Collect statuses in parallel
    let mut statuses = children
        .par_iter()
        .filter_map(|path| collect_repo_status(path).transpose())
        .collect::<Result<Vec<_>>>()?;

    // Sort by repository name
    statuses.sort_by(|(a, _), (b, _)| a.cmp(b));

    Ok(statuses)
}

fn collect_repo_status(repo_path: &Path)
-> Result<Option<(String, RepoStatus)>>
{
    // Skip non-git directories
    if !repo_path.join(".git").exists()
    {
        return Ok(None);
    }

    // Get repository name
    let repo_name = repo_path
        .file_name()
        .and_then(|name| name.to_str())
        .context("repository path has no valid UTF-8 file name")?
        .to_owned();

    let mut staged_changes = Vec::new();
    let mut unstaged_changes = Vec::new();
    let mut untracked_files = Vec::new();

    // Read porcelain status
    let status_output = git_output(repo_path, &[
        "status",
        "--porcelain=v1",
        "-z",
        "--branch",
        "--no-ahead-behind"
    ])
    .with_context(|| {
        format!("failed to read git status for '{}'", repo_path.display())
    })?;
    let mut records = status_output
        .split(|byte| *byte == 0)
        .filter(|record| !record.is_empty());

    // Parse branch header
    let branch_header =
        records.next().context("git status did not return a branch header")?;
    let (head, initial) = parse_branch_header(repo_path, branch_header)?;

    // Parse path records
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

        // Skip the old path record for renamed or copied files
        if matches!(index_status, 'R' | 'C')
            || matches!(worktree_status, 'R' | 'C')
        {
            let _old_path = records.next();
        }

        // Track untracked files separately
        if index_status == '?' && worktree_status == '?'
        {
            untracked_files
                .push(FileEntry { path, change: FileChange::NewFile });
            continue;
        }

        // Treat intent-to-add as staged for VMR status output
        if index_status == ' ' && worktree_status == 'A'
        {
            staged_changes
                .push(FileEntry { path, change: FileChange::NewFile });
            continue;
        }

        // Add index and worktree changes
        if let Some(change) = parse_status_change(index_status)
        {
            staged_changes.push(FileEntry { path: path.clone(), change });
        }
        if let Some(change) = parse_status_change(worktree_status)
        {
            unstaged_changes.push(FileEntry { path, change });
        }
    }

    Ok(Some((repo_name, RepoStatus {
        head,
        initial,
        staged_changes,
        unstaged_changes,
        untracked_files
    })))
}

fn git_output(repo_path: &Path, args: &[&str]) -> Result<Vec<u8>>
{
    // Run git command
    let output = Command::new("git")
        .arg("--no-optional-locks")
        .arg("-C")
        .arg(repo_path)
        .args(args)
        .output()
        .with_context(|| {
            format!("failed to invoke git for '{}'", repo_path.display())
        })?;

    // Convert git failure into anyhow error
    if !output.status.success()
    {
        bail!(
            "git {} failed for '{}': {}",
            args.join(" "),
            repo_path.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    Ok(output.stdout)
}

fn parse_branch_header(repo_path: &Path, header: &[u8])
-> Result<(Head, bool)>
{
    // Decode and validate branch header
    let header = String::from_utf8_lossy(header);
    let header = header
        .strip_prefix("## ")
        .context("git status branch header had unexpected format")?;

    // Detect unborn branch
    if let Some(branch) = header.strip_prefix("No commits yet on ")
    {
        return Ok((Head::Branch(branch.to_owned()), true));
    }

    // Detect detached HEAD
    if header == "HEAD (no branch)" || header.starts_with("HEAD detached")
    {
        let hash = String::from_utf8_lossy(&git_output(repo_path, &[
            "rev-parse",
            "--short",
            "HEAD"
        ])?)
        .trim()
        .to_owned();
        return Ok((Head::Detached(hash), false));
    }

    // Parse branch name
    let branch = header.split("...").next().unwrap_or(header).to_owned();
    Ok((Head::Branch(branch), false))
}

fn parse_status_change(status: char) -> Option<FileChange>
{
    // Map porcelain status code
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

fn render_status(
    statuses: &[(String, RepoStatus)],
    working_dir: &Path,
    vmr_root: &Path
) -> String
{
    let mut output = String::new();
    let styles = StatusStyles::new();

    // Count regular committed branches
    let regular_branch_count = statuses
        .iter()
        .filter_map(|(_, status)| match &status.head
        {
            Head::Branch(branch) if !status.initial => Some(branch.as_str()),
            Head::Detached(_) => None,
            Head::Branch(_) => None
        })
        .collect::<BTreeSet<_>>()
        .len();

    let mut branch_groups: BTreeMap<(&str, bool), Vec<(&str, &RepoStatus)>> =
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
            Head::Detached(hash) =>
                detached.push((repo.as_str(), hash.as_str(), status)),
        }
    }

    // Render branch groups
    for ((branch, initial), repos) in branch_groups
    {
        let repo_names =
            repos.iter().map(|(repo, _)| *repo).collect::<Vec<_>>();
        if !initial && regular_branch_count == 1 && detached.is_empty()
        {
            output.push_str(&format!("On branch {}\n", branch));
        }
        else
        {
            output.push_str(&format!(
                "On branch {} ({})\n",
                branch,
                repo_names.join(", ")
            ));
        }

        render_group(&mut output, &repos, working_dir, vmr_root, &styles);
    }

    // Render detached repositories
    for (repo, hash, status) in detached
    {
        output.push_str(&format!("HEAD detached at {} ({repo})\n", hash));
        render_group(
            &mut output,
            &[(repo, status)],
            working_dir,
            vmr_root,
            &styles
        );
    }

    output
}

fn render_group(
    output: &mut String,
    repos: &[(&str, &RepoStatus)],
    working_dir: &Path,
    vmr_root: &Path,
    styles: &StatusStyles
)
{
    let context = RenderContext { repos, working_dir, vmr_root };

    // Render initial commit notice
    if repos.iter().any(|(_, status)| status.initial)
    {
        output.push_str("No commits yet.");
        output.push('\n');
    }

    // Render change sections
    let has_staged = render_paths(
        output,
        "Changes to be committed:",
        &["  (use \"git vmr restore --staged <file>...\" to unstage)"],
        &context,
        |status| &status.staged_changes,
        styles.staged
    );
    let has_unstaged = render_paths(
        output,
        "Changes not staged for commit:",
        &[
            "  (use \"git vmr add <file>...\" to update what will be committed)",
            "  (use \"git vmr restore <file>...\" to discard changes in working directory)"
        ],
        &context,
        |status| &status.unstaged_changes,
        styles.changed
    );
    let has_untracked = render_paths(
        output,
        "Untracked files:",
        &[
            "  (use \"git vmr add <file>...\" to include in what will be committed)"
        ],
        &context,
        |status| &status.untracked_files,
        styles.changed
    );

    // Render clean summary
    if has_staged || has_unstaged || has_untracked
    {
        output.push('\n');
    }
    else if repos.iter().all(|(_, status)| !status.initial)
    {
        output.push_str("nothing to commit, working tree clean\n");
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
    hints: &[&str],
    context: &RenderContext,
    paths: fn(&RepoStatus) -> &Vec<FileEntry>,
    style: StatusStyle
) -> bool
{
    // Collect paths for this group
    let mut rendered: Vec<(String, FileChange)> = Vec::new();
    for (repo, status) in context.repos
    {
        let repo_path = context.vmr_root.join(repo);
        let repo_prefix = pathdiff::diff_paths(&repo_path, context.working_dir)
            .unwrap_or(repo_path);
        for entry in paths(status)
        {
            let rel = repo_prefix.join(&entry.path);
            rendered.push((rel.display().to_string(), entry.change));
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
    use std::process::Command;

    #[test]
    fn renders_single_branch_without_repo_list()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let statuses = vec![
            ("backend".to_owned(), repo_status("main")),
            ("frontend".to_owned(), repo_status("main")),
        ];

        // Act
        let output = render_status(&statuses, tmp.path(), tmp.path());

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
        let statuses = vec![("backend".to_owned(), repo_status("main"))];

        // Act
        let output = render_status(&statuses, tmp.path(), tmp.path());

        // Assert
        assert!(output.contains("On branch main"));
        assert!(output.contains("nothing to commit, working tree clean"));
    }

    #[test]
    fn renders_diverged_branches_with_repo_names()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let statuses = vec![
            ("backend".to_owned(), repo_status("main")),
            ("frontend".to_owned(), repo_status("feature/auth")),
        ];

        // Act
        let output = render_status(&statuses, tmp.path(), tmp.path());

        // Assert
        assert!(output.contains("main"));
        assert!(output.contains("(backend)"));
        assert!(output.contains("feature/auth"));
        assert!(output.contains("(frontend)"));
    }

    #[test]
    fn renders_detached_head()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let mut status = repo_status("main");
        status.head = Head::Detached("a1b2c3d".to_owned());
        let statuses = vec![("tools".to_owned(), status)];

        // Act
        let output = render_status(&statuses, tmp.path(), tmp.path());

        // Assert
        assert!(output.contains("HEAD detached at "));
        assert!(output.contains("a1b2c3d"));
        assert!(output.contains("(tools)"));
        assert!(!output.contains("\u{1b}[33m"));
    }

    #[test]
    fn renders_no_commits()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let mut status = repo_status("master");
        status.initial = true;
        let statuses = vec![("new-repo".to_owned(), status)];

        // Act
        let output = render_status(&statuses, tmp.path(), tmp.path());

        // Assert
        assert!(output.contains("master"));
        assert!(output.contains("(new-repo)"));
        assert!(output.contains("No commits yet."));
    }

    #[test]
    fn renders_initial_repos_separately_from_committed_repos_on_same_branch()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let mut initial = repo_status("master");
        initial.initial = true;
        let statuses = vec![
            ("committed".to_owned(), repo_status("master")),
            ("new-repo".to_owned(), initial),
        ];

        // Act
        let output = render_status(&statuses, tmp.path(), tmp.path());

        // Assert
        assert!(output.contains(
            "On branch master\nnothing to commit, working tree clean"
        ));
        assert!(
            output.contains("On branch master (new-repo)\nNo commits yet.")
        );
        assert!(!output.contains("On branch master (committed, new-repo)"));
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
        let statuses = vec![("backend".to_owned(), status)];

        // Act
        let output = render_status(&statuses, &working_dir, tmp.path());

        // Assert
        assert!(output.contains("../backend/src/main.rs"));
        assert!(output.contains("\u{1b}[31m"));
    }

    #[test]
    fn collects_git_repos_and_skips_non_git_dirs()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir(tmp.path().join(".gitvmr")).unwrap();
        fs::create_dir(tmp.path().join("docs")).unwrap();
        init_git_repo(&tmp.path().join("backend"));

        // Act
        let statuses = collect_statuses(tmp.path()).unwrap();

        // Assert
        assert_eq!(statuses.len(), 1);
        assert_eq!(statuses[0].0, "backend");
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
