use crate::cli::CliContext;
use crate::git::{Git, stderr};
use crate::render::Rendered;
use crate::vmr::{
    self, Declaration, Gitignore, Layout, MANIFEST, Manifest, ManifestEditor,
    Membership, RepoPath, RepoUrl
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

    // Reinitialize, refuse or convert by what the target already holds
    match Layout::of(&target_dir)
    {
        // An existing manifest is left alone: declaring later is `add`'s job
        Layout::Root | Layout::ManifestWithoutGit =>
        {
            git.init_root(&target_dir)?;
            rendered.stdout = format!(
                "Reinitialized existing virtual monorepo in {}\n",
                target_dir.display()
            );
            return Ok(rendered);
        }

        Layout::LegacyWorktreeRoot =>
            return Err(vmr::legacy_worktree_root_error(&target_dir)),

        // Convert a legacy VMR, whose marker directory held only config
        Layout::LegacyVmr =>
        {
            fs::remove_dir_all(&manifest_path).with_context(|| {
                format!("fatal: failed to remove '{}'", manifest_path.display())
            })?;
            rendered.stdout.push_str("Removed legacy .gitvmr directory\n");
        }

        Layout::NoManifest =>
        {}
    }

    // Make the target a root repo
    git.init_root(&target_dir)?;
    let _ = writeln!(
        rendered.stdout,
        "Initialized virtual monorepo in {}",
        target_dir.display()
    );

    // Backfill the manifest from the Git repositories already present, all
    // of them undeclared against an empty manifest
    let membership = Membership::read(&target_dir, &Manifest::default())?;
    let mut editor = ManifestEditor::new();
    let mut declared = Vec::new();
    for name in membership.undeclared()
    {
        match backfill(git, &target_dir, name)?
        {
            Backfill::Declare(declaration) =>
            {
                editor.declare(&declaration);
                let _ = writeln!(rendered.stdout, "Declared '{name}'");
                declared.push(declaration.path);
            }
            Backfill::Skip(reason) =>
            {
                let _ = writeln!(
                    rendered.stderr,
                    "warning: skipped '{name}': {reason}"
                );
            }
        }
    }

    // Write manifest
    fs::write(&manifest_path, editor.to_string()).with_context(|| {
        format!("fatal: failed to write '{}'", manifest_path.display())
    })?;

    // Ignore the declared child repos in the root repo
    let mut files = MANIFEST.to_owned();
    if !declared.is_empty()
    {
        Gitignore::read(&target_dir)
            .ignore(declared.iter().map(RepoPath::as_str))?;
        files.push_str(" .gitignore");
    }

    // Suggest next steps: until its first commit the root repo is unborn
    let _ = write!(
        rendered.stdout,
        "\nNext, commit the manifest and add a remote:\n  \
         git add {files}\n  \
         git commit -m \"Create virtual monorepo\"\n  \
         git remote add origin <url>\n"
    );

    Ok(rendered)
}

/// What backfilling does with a Git repository found in the target.
enum Backfill
{
    Declare(Declaration),
    Skip(String)
}

/// Decides whether the repo at `root/name` can be declared: it needs a
/// declarable name, an `origin` URL to declare, and a directory the root
/// repo does not already track files under.
fn backfill(git: &Git, root: &Path, name: &str) -> Result<Backfill>
{
    // Require a declarable name
    let path = match RepoPath::parse(name.to_owned())
    {
        Ok(path) => path,
        Err(reason) => return Ok(Backfill::Skip(reason))
    };

    // Require a non-empty origin URL
    let Some(url) = git.origin_url(&root.join(name))?
    else
    {
        return Ok(Backfill::Skip("it has no 'origin' remote".to_owned()));
    };
    let Ok(url) = RepoUrl::parse(url)
    else
    {
        return Ok(Backfill::Skip("its 'origin' URL is empty".to_owned()));
    };

    // Require a directory the root repo does not track files under
    if git.tracks_under(root, name)?
    {
        return Ok(Backfill::Skip(
            "the root repo tracks files under it".to_owned()
        ));
    }

    Ok(Backfill::Declare(Declaration { name: name.to_owned(), path, url }))
}

impl Git
{
    /// Makes `dir` a Git repository unless it already is one.
    fn init_root(&self, dir: &Path) -> Result<()>
    {
        // Keep an existing Git repository
        if dir.join(".git").exists()
        {
            return Ok(());
        }

        // Run git init
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
        // Read the configured URL
        let output =
            self.output(repo, ["config", "--get", "remote.origin.url"])?;

        // Exit code 1 means the key is unset
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
