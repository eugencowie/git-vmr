use crate::cli::{CliContext, SilentError};
use crate::git::report::{FailureReport, SuccessReport, command_result};
use crate::git::{Git, GitCommandResult, failure_message};
use crate::render::{self, Rendered};
use crate::vmr::{Declaration, Layout, Membership, Repo, Vmr};
use anyhow::{Result, anyhow, bail};
use path_clean::PathClean;
use rayon::prelude::*;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;

pub fn run(context: &CliContext, args: &CloneArgs) -> Result<Rendered>
{
    match &args.repository
    {
        // With a repository, clone a whole VMR from it
        Some(repository) =>
            clone_vmr(context, repository, args.directory.as_deref()),

        // Without a repository, fill in the VMR the working dir is in
        None => clone_children(&context.git, &Vmr::find(&context.working_dir)?)
    }
}

/// Clones the root repo from `repository` into `directory`, then every
/// child repo its manifest declares.
fn clone_vmr(
    context: &CliContext,
    repository: &str,
    directory: Option<&Path>
) -> Result<Rendered>
{
    // Name the directory git would guess, so the root repo lands where it
    // is looked for
    let directory = match directory
    {
        Some(directory) => directory.to_path_buf(),
        None => PathBuf::from(guess_directory(repository).ok_or_else(|| {
            anyhow!(
                "fatal: no directory name could be guessed from \
                 '{repository}'\n\
                 hint: specify a directory on the command line"
            )
        })?)
    };

    // Clone root repo interactively; git reports its own errors
    let status =
        context.git.clone_root(&context.working_dir, repository, &directory)?;
    if !status.success()
    {
        return Err(SilentError.into());
    }

    // Require a manifest, keeping the clone without one
    let root = context.working_dir.join(&directory).clean();
    if Layout::of(&root) != Layout::Root
    {
        bail!(
            "fatal: cloned '{}', but it has no .gitvmr manifest\n\
             hint: the root repo's default branch may not have one",
            directory.display()
        );
    }

    // Clone child repos
    clone_children(&context.git, &Vmr { path: root })
}

/// Clones every declared child repo of `vmr` that is not cloned yet, in
/// parallel, reporting one repo outcome each against every member repo.
fn clone_children(git: &Git, vmr: &Vmr) -> Result<Rendered>
{
    // Find the declared child repos that are not cloned
    let membership = Membership::read(&vmr.path, &vmr.manifest()?)?;
    let missing = membership.missing();
    if missing.is_empty()
    {
        return Ok(Rendered::default());
    }

    // Read the root repo's URL, needed only to resolve relative URLs
    let root_url =
        if missing.iter().any(|declaration| declaration.url.is_relative())
        {
            git.origin_url(&vmr.path)?
        }
        else
        {
            None
        };

    // Clone them in parallel
    let outcomes = missing
        .par_iter()
        .map(|declaration| {
            git.clone_child(&vmr.path, declaration, root_url.as_deref())
        })
        .collect();

    // Report against every member repo, the new ones included
    render::outcomes(
        outcomes,
        membership
            .members()
            .iter()
            .map(|repo| repo.path.as_str())
            .chain(missing.iter().map(|declaration| declaration.path.as_str()))
    )
}

/// Clone a virtual monorepo into a new directory
#[derive(clap::Args)]
pub struct CloneArgs
{
    /// The (possibly remote) <repository> to clone the root repo from.
    /// Without one, clones the child repos the current VMR declares but
    /// has not cloned
    #[arg(value_name = "repository")]
    pub repository: Option<String>,

    /// The name of a new directory to clone into
    #[arg(value_name = "directory")]
    pub directory: Option<PathBuf>
}

/// The directory `git clone` would create for `repository`, or `None` when
/// it could guess none: the URL's last path component, without
/// authentication data, a host-only port, or a trailing `.git` or `/.git`,
/// with runs of whitespace and control characters collapsed to one space.
/// A port of git's `git_url_basename`, except that a local bundle keeps its
/// `.bundle` suffix.
fn guess_directory(repository: &str) -> Option<String>
{
    // Git's character classes: C's isspace, and native dir separators
    let is_space =
        |c: char| matches!(c, ' ' | '\t' | '\n' | '\x0b' | '\x0c' | '\r');
    let is_dir_sep = |c: char| c == '/' || (cfg!(windows) && c == '\\');

    // Skip scheme, then authentication data up to the last `@` in the host
    let mut url =
        repository.split_once("://").map_or(repository, |(_, rest)| rest);
    let host = url.split(is_dir_sep).next().unwrap_or_default();
    if let Some(at) = host.rfind('@')
    {
        url = &url[at + 1..];
    }

    // Strip trailing separators and whitespace, then `/.git`
    url = url.trim_end_matches(|c: char| is_dir_sep(c) || is_space(c));
    if url.len() > 5
        && let Some(stripped) = url.strip_suffix(".git")
        && stripped.ends_with(is_dir_sep)
    {
        url = stripped.trim_end_matches(is_dir_sep);
    }

    // Strip a port from a bare host, as in `host:2222`
    if !url.contains('/')
        && let Some((host, port)) = url.rsplit_once(':')
        && port.bytes().all(|byte| byte.is_ascii_digit())
    {
        url = host;
    }

    // Take the last component, treating colons as separators too, without
    // `.git`
    let name = url
        .rsplit(|c: char| is_dir_sep(c) || c == ':')
        .next()
        .unwrap_or_default();
    let name = name.strip_suffix(".git").unwrap_or(name);

    // Collapse whitespace and control characters into single spaces
    let name = name
        .split(|c: char| c < ' ' || is_space(c))
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    (!name.is_empty()).then_some(name)
}

impl Git
{
    /// Runs git clone with inherited stdio so credential prompts and
    /// progress reach the user. Git reports its own errors on stderr; the
    /// caller decides what a non-success exit status means.
    fn clone_root(
        &self,
        working_dir: &Path,
        repository: &str,
        directory: &Path
    ) -> Result<ExitStatus>
    {
        self.interactive(working_dir, &[
            OsString::from("clone"),
            OsString::from(repository),
            directory.as_os_str().to_owned()
        ])
    }

    /// Clones the child repo `declaration` declares at the VMR root `root`,
    /// resolving a relative URL against `root_url`.
    fn clone_child(
        &self,
        root: &Path,
        declaration: &Declaration,
        root_url: Option<&str>
    ) -> GitCommandResult
    {
        // Resolve URL
        let repo = Repo::child(root, declaration);
        let url = match declaration.url.resolve(root_url)
        {
            Ok(url) => url,
            Err(error) =>
                return Ok(failure_message(&repo.path, format!("{error:#}"))),
        };

        // Run git clone in the VMR root, capturing its output
        let output = self.output(root, ["clone", url.as_str(), &repo.path])?;

        // Report outcome
        command_result(
            &repo,
            &output,
            SuccessReport::fixed("Cloned".to_owned()),
            FailureReport::last_stderr_line("git clone failed")
        )
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::git::{Git, Invocation, ScriptedFake};
    use crate::render::Failed;
    use crate::test_support::root_repo;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    const ORIGIN: [&str; 3] = ["config", "--get", "remote.origin.url"];

    fn clone_args(
        repository: Option<&str>,
        directory: Option<&str>
    ) -> CloneArgs
    {
        CloneArgs {
            repository: repository.map(str::to_owned),
            directory: directory.map(PathBuf::from)
        }
    }

    fn invocation(path: &Path, args: &[&str]) -> Invocation
    {
        Invocation {
            path: path.to_path_buf(),
            args: args.iter().map(Into::into).collect()
        }
    }

    /// The child clones `fake` ran in the VMR root `root`, sorted, since
    /// they run in parallel.
    fn child_clones(fake: &ScriptedFake, root: &Path) -> Vec<Invocation>
    {
        let mut clones = fake
            .calls()
            .into_iter()
            .filter(|invocation| {
                invocation.path == root && invocation.args[0] == "clone"
            })
            .collect::<Vec<_>>();
        clones.sort_by(|a, b| a.args.cmp(&b.args));
        clones
    }

    #[test]
    fn guesses_the_directory_git_would_clone_into()
    {
        for (repository, directory) in [
            ("https://example.com/repo", "repo"),
            ("https://example.com/repo.git", "repo"),
            ("https://example.com/repo.git/", "repo"),
            ("/path/to/repo/.git", "repo"),
            ("/path/to/repo//.git/", "repo"),
            ("host.xz:foo/.git", "foo"),
            ("git@host:org/repo.git", "repo"),
            ("foo:bar.git", "bar"),
            ("/foo/bar:2222.git", "2222"),
            ("ssh://user@host:2222", "host"),
            ("ssh://user:pw@host", "host"),
            ("https://host/a \t b\u{1}c .git", "a b c")
        ]
        {
            // Act
            let guessed = guess_directory(repository);

            // Assert
            assert_eq!(guessed.as_deref(), Some(directory), "{repository}");
        }
    }

    #[test]
    fn guesses_no_directory_for_a_repository_without_a_name()
    {
        for repository in ["", "/", ".git", "/.git"]
        {
            // Act
            let guessed = guess_directory(repository);

            // Assert
            assert_eq!(guessed, None, "{repository}");
        }
    }

    #[test]
    fn clones_the_root_repo_then_every_declared_child_repo()
    {
        // Arrange: the root repo as the interactive clone leaves it
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("copy");
        root_repo(&root, &["backend", "frontend"], &["backend", "frontend"]);
        let fake = Arc::new(
            ScriptedFake::new()
                .on_interactive(
                    ["clone", "https://host/org/vmr.git", "copy"],
                    0
                )
                .on(ORIGIN, 0, "https://host/org/vmr.git\n", "")
                .on(
                    ["clone", "https://host/org/backend.git", "backend"],
                    0,
                    "",
                    ""
                )
                .on(
                    ["clone", "https://host/org/frontend.git", "frontend"],
                    0,
                    "",
                    ""
                )
        );
        let git = Git::with(Arc::clone(&fake));

        // Act
        let rendered = run(
            &CliContext::for_tests(tmp.path(), git),
            &clone_args(Some("https://host/org/vmr.git"), Some("copy"))
        )
        .unwrap();

        // Assert
        assert_eq!(
            fake.calls()[0],
            invocation(tmp.path(), &[
                "clone",
                "https://host/org/vmr.git",
                "copy"
            ])
        );
        assert_eq!(child_clones(&fake, &root), [
            invocation(&root, &[
                "clone",
                "https://host/org/backend.git",
                "backend"
            ]),
            invocation(&root, &[
                "clone",
                "https://host/org/frontend.git",
                "frontend"
            ])
        ]);
        assert_eq!(
            rendered.stdout,
            "Cloned \x1b[90m(backend, frontend)\x1b[0m\n"
        );
    }

    #[test]
    fn clones_the_root_repo_into_the_directory_git_would_guess()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        root_repo(&tmp.path().join("vmr"), &[], &[]);
        let fake =
            Arc::new(ScriptedFake::new().on_interactive(
                ["clone", "https://host/org/vmr.git", "vmr"],
                0
            ));
        let git = Git::with(Arc::clone(&fake));

        // Act
        let rendered = run(
            &CliContext::for_tests(tmp.path(), git),
            &clone_args(Some("https://host/org/vmr.git"), None)
        )
        .unwrap();

        // Assert
        assert_eq!(fake.calls().len(), 1);
        assert_eq!(rendered.stdout, "");
    }

    #[test]
    fn clone_without_a_manifest_keeps_the_root_repo_and_fails()
    {
        // Arrange: the cloned root repo has no manifest
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("copy/.git")).unwrap();
        let git =
            Git::with(ScriptedFake::new().on_interactive(
                ["clone", "https://host/org/repo.git", "copy"],
                0
            ));

        // Act
        let err = run(
            &CliContext::for_tests(tmp.path(), git),
            &clone_args(Some("https://host/org/repo.git"), Some("copy"))
        )
        .unwrap_err();

        // Assert
        assert_eq!(
            format!("{err:#}"),
            "fatal: cloned 'copy', but it has no .gitvmr manifest\n\
             hint: the root repo's default branch may not have one"
        );
        assert!(tmp.path().join("copy/.git").exists());
    }

    #[test]
    fn a_failed_child_clone_fails_the_command_after_the_others_clone()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("copy");
        root_repo(&root, &["backend", "frontend"], &["backend", "frontend"]);
        let fake = Arc::new(
            ScriptedFake::new()
                .on_interactive(["clone", "https://host/vmr.git", "copy"], 0)
                .on(ORIGIN, 0, "https://host/vmr.git\n", "")
                .on(["clone", "https://host/backend.git", "backend"], 0, "", "")
                .on(
                    ["clone", "https://host/frontend.git", "frontend"],
                    128,
                    "",
                    "Cloning into 'frontend'...\nfatal: repository not found\n"
                )
        );
        let git = Git::with(Arc::clone(&fake));

        // Act
        let err = run(
            &CliContext::for_tests(tmp.path(), git),
            &clone_args(Some("https://host/vmr.git"), Some("copy"))
        )
        .unwrap_err();

        // Assert
        let failed = err.downcast::<Failed>().unwrap();
        assert_eq!(failed.rendered.stdout, "Cloned \x1b[90m(backend)\x1b[0m\n");
        assert_eq!(
            failed.message,
            "fatal: repository not found \x1b[90m(frontend)\x1b[0m"
        );
    }

    #[test]
    fn a_relative_url_without_a_root_origin_fails_that_child_only()
    {
        // Arrange: `backend` declares a relative URL, the root has no origin
        let tmp = tempfile::tempdir().unwrap();
        root_repo(tmp.path(), &["backend"], &["backend"]);
        let fake = Arc::new(ScriptedFake::new().on(ORIGIN, 1, "", ""));
        let git = Git::with(Arc::clone(&fake));

        // Act
        let err = run(
            &CliContext::for_tests(tmp.path(), git),
            &clone_args(None, None)
        )
        .unwrap_err();

        // Assert
        assert_eq!(
            err.downcast::<Failed>().unwrap().message,
            "cannot resolve relative URL '../backend.git': the root repo has \
             no 'origin' remote \x1b[90m(backend)\x1b[0m"
        );
        assert!(child_clones(&fake, tmp.path()).is_empty());
    }

    #[test]
    fn clone_without_a_repository_clones_the_missing_child_repos()
    {
        // Arrange: `backend` is cloned, `api` is not
        let tmp = tempfile::tempdir().unwrap();
        root_repo(tmp.path(), &["api", "backend"], &["api", "backend"]);
        fs::create_dir_all(tmp.path().join("backend/.git")).unwrap();
        let fake = Arc::new(
            ScriptedFake::new().on(ORIGIN, 0, "/srv/git/vmr\n", "").on(
                ["clone", "/srv/git/api.git", "api"],
                0,
                "",
                ""
            )
        );
        let git = Git::with(Arc::clone(&fake));

        // Act
        let rendered = run(
            &CliContext::for_tests(&tmp.path().join("backend"), git),
            &clone_args(None, None)
        )
        .unwrap();

        // Assert
        assert_eq!(child_clones(&fake, tmp.path()), [invocation(
            tmp.path(),
            &["clone", "/srv/git/api.git", "api"]
        )]);
        assert_eq!(rendered.stdout, "Cloned \x1b[90m(api)\x1b[0m\n");
    }

    #[test]
    fn clone_without_a_repository_does_nothing_when_nothing_is_missing()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        root_repo(tmp.path(), &["backend"], &["backend"]);
        fs::create_dir_all(tmp.path().join("backend/.git")).unwrap();
        let fake = Arc::new(ScriptedFake::new());
        let git = Git::with(Arc::clone(&fake));

        // Act
        let rendered = run(
            &CliContext::for_tests(tmp.path(), git),
            &clone_args(None, None)
        )
        .unwrap();

        // Assert
        assert!(fake.calls().is_empty());
        assert_eq!(rendered.stdout, "");
    }

    #[test]
    fn clone_failure_is_silent_because_git_already_reported_it()
    {
        // Arrange
        let fake = ScriptedFake::new()
            .on_interactive(["clone", "https://example.com/repo", "repo"], 128);
        let git = Git::with(fake);

        // Act
        let error = run(
            &CliContext::for_tests(Path::new("/vmr"), git),
            &clone_args(Some("https://example.com/repo"), None)
        )
        .unwrap_err();

        // Assert
        assert!(error.is::<SilentError>());
    }
}
