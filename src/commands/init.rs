use crate::cli::CliContext;
use crate::git::{Git, stderr};
use crate::render::Rendered;
use crate::vmr::{
    self, Declaration, MANIFEST, ManifestEditor, Vmr, validate_path
};
use anyhow::{Context, Result, bail};
use path_clean::PathClean;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

/// Create an empty virtual monorepo or reinitialize an existing one
#[derive(clap::Args)]
pub struct InitArgs
{
    /// If you provide a directory, the command is run inside it. If this
    /// directory does not exist, it will be created
    #[arg(value_name = "directory")]
    pub directory: Option<PathBuf>
}

pub fn run(context: &CliContext, args: &InitArgs) -> Result<Rendered>
{
    let working_dir = &context.working_dir;

    // Resolve init target directory
    let target_dir = match args.directory.as_deref()
    {
        // Resolve provided directory against the effective working directory
        Some(directory) => working_dir.join(directory).clean(),

        // If none provided, use the effective working directory
        None => working_dir.to_path_buf()
    };

    // Ensure the target path is a directory
    if target_dir.exists() && !target_dir.is_dir()
    {
        bail!(
            "fatal: cannot initialize '{}': Not a directory",
            target_dir.display()
        );
    }

    // Create target directory
    fs::create_dir_all(&target_dir).with_context(|| {
        format!(
            "fatal: failed to create init directory '{}'",
            target_dir.display()
        )
    })?;

    let git = &context.git;
    let manifest_path = target_dir.join(MANIFEST);
    let mut rendered = Rendered::default();

    // An existing manifest is left alone: declaring later is `add`'s job
    if manifest_path.is_file()
    {
        Vmr::reject_old_worktree_root(&target_dir)?;
        git.init_root(&target_dir)?;
        rendered.stdout = format!(
            "Reinitialized existing virtual monorepo in {}\n",
            target_dir.display()
        );
        return Ok(rendered);
    }

    // Convert an old-style VMR, whose marker directory held only config
    if manifest_path.is_dir()
    {
        fs::remove_dir_all(&manifest_path).with_context(|| {
            format!("fatal: failed to remove '{}'", manifest_path.display())
        })?;
        rendered.stdout.push_str("Removed old-style .gitvmr directory\n");
    }

    git.init_root(&target_dir)?;
    let _ = writeln!(
        rendered.stdout,
        "Initialized virtual monorepo in {}",
        target_dir.display()
    );

    // Backfill the manifest from the Git repositories already present
    let mut editor = ManifestEditor::new();
    let mut declared = Vec::new();
    for path in first_level_repos(&target_dir)?
    {
        if let Err(reason) = validate_path(&path)
        {
            let _ = writeln!(
                rendered.stderr,
                "warning: skipped '{path}': {reason}"
            );
            continue;
        }

        let skipped = match git.origin_url(&target_dir.join(&path))?
        {
            None => Some("it has no 'origin' remote"),
            Some(_) if git.tracks_under(&target_dir, &path)? =>
                Some("the root repo tracks files under it"),
            Some(url) =>
            {
                editor.declare(&Declaration {
                    name: path.clone(),
                    path: path.clone(),
                    url
                });
                let _ = writeln!(rendered.stdout, "Declared '{path}'");
                declared.push(path.clone());
                None
            }
        };
        if let Some(reason) = skipped
        {
            let _ = writeln!(
                rendered.stderr,
                "warning: skipped '{path}': {reason}"
            );
        }
    }

    fs::write(&manifest_path, editor.to_string()).with_context(|| {
        format!("fatal: failed to write '{}'", manifest_path.display())
    })?;
    let mut files = MANIFEST.to_owned();
    if !declared.is_empty()
    {
        ignore(&target_dir, &declared)?;
        files.push_str(" .gitignore");
    }

    // Until its first commit the root repo is unborn
    let _ = write!(
        rendered.stdout,
        "\nNext, commit the manifest and add a remote:\n  \
         git add {files}\n  \
         git commit -m \"Create virtual monorepo\"\n  \
         git remote add origin <url>\n"
    );

    Ok(rendered)
}

/// The names of the target's immediate subdirectories that are Git
/// repositories, sorted. Names that are not valid UTF-8 cannot be declared
/// and are left out.
fn first_level_repos(target_dir: &Path) -> Result<Vec<String>>
{
    let mut repos = fs::read_dir(target_dir)
        .with_context(|| {
            format!("fatal: failed to read '{}'", target_dir.display())
        })?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .filter(|entry| entry.path().join(".git").exists())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| name != ".git")
        .collect::<Vec<_>>();
    repos.sort();

    Ok(repos)
}

/// Ignores child repos in the root repo: appends an anchored `/<path>/`
/// line to its `.gitignore` for each path that lacks one, keeping the rest
/// of the file as it is.
///
/// Not `.git/info/exclude`: that file is shared by every worktree of the
/// root repo while manifests are per branch, so one worktree's ignore lines
/// would be another's too.
fn ignore(root: &Path, paths: &[String]) -> Result<()>
{
    let mut gitignore = vmr::read_gitignore(root);
    let missing = paths
        .iter()
        .filter(|path| !vmr::has_ignore_line(&gitignore, path))
        .collect::<Vec<_>>();
    if missing.is_empty()
    {
        return Ok(());
    }

    if !gitignore.is_empty() && !gitignore.ends_with('\n')
    {
        gitignore.push('\n');
    }
    for path in missing
    {
        let _ = writeln!(gitignore, "/{path}/");
    }

    let gitignore_path = root.join(".gitignore");
    fs::write(&gitignore_path, gitignore).with_context(|| {
        format!("fatal: failed to write '{}'", gitignore_path.display())
    })
}

impl Git
{
    /// Makes `dir` a Git repository unless it already is one.
    fn init_root(&self, dir: &Path) -> Result<()>
    {
        if dir.join(".git").exists()
        {
            return Ok(());
        }

        let output = self.output(dir, ["init", "--quiet"])?;
        if !output.status.success()
        {
            bail!("fatal: git init failed: {}", stderr(&output));
        }

        Ok(())
    }

    /// Whether the root repo tracks anything at or under `path` — a
    /// directory both repos would claim, since ignore rules do not untrack
    /// files.
    fn tracks_under(&self, root: &Path, path: &str) -> Result<bool>
    {
        let tracked = self.stdout(root, [
            "--literal-pathspecs",
            "ls-files",
            "-z",
            "--",
            path
        ])?;

        Ok(!tracked.is_empty())
    }

    /// The repo's `origin` URL as configured, if it has one.
    fn origin_url(&self, repo: &Path) -> Result<Option<String>>
    {
        let output =
            self.output(repo, ["config", "--get", "remote.origin.url"])?;

        match output.status.code()
        {
            Some(0) => Ok(Some(
                String::from_utf8_lossy(&output.stdout).trim().to_owned()
            )),
            Some(1) => Ok(None),
            _ => bail!(
                "fatal: failed to read the 'origin' remote of '{}': {}",
                repo.display(),
                stderr(&output)
            )
        }
    }
}
