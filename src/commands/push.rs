use crate::cli::CliContext;
use crate::render::Rendered;
use crate::workspace::Workspace;
use anyhow::Result;

pub fn run(
    workspace: &Workspace,
    _context: &CliContext,
    args: &PushArgs
) -> Result<Rendered>
{
    // Push in each child repository whose push classifies as useful
    workspace.run(|git, repo| {
        if args.repository.is_none()
            && args.refspecs.is_empty()
            && let BarePush::Skip(reason) =
                git.classify_bare_push(&repo.path)?
        {
            return Ok(skip_message(&repo.name, reason));
        }

        git.push(repo, args)
    })
}

use crate::git::report::{
    FailureReport, OnEmpty, Streams, SuccessReport, command_result
};
use crate::git::{Git, GitCommandResult, Head, skip_message, stderr};
use crate::vmr::Repo;
use anyhow::bail;
use std::ffi::OsString;
use std::path::Path;

/// Update remote refs along with associated objects
#[derive(clap::Args)]
pub struct PushArgs
{
    /// The "remote" repository that is the destination of a push operation
    #[arg(value_name = "repository")]
    pub repository: Option<String>,

    /// Specify what destination ref to update with what source object
    #[arg(value_name = "refspec")]
    pub refspecs: Vec<String>
}

/// How a bare push classified: attempt it (delegating everything else to
/// git) or deliberately withhold it, with the reason to report.
enum BarePush
{
    Delegate,
    Skip(String)
}

/// Where a bare push would land, resolved from config the way `git push`
/// would resolve it — or the classification when resolution ends early.
enum Destination
{
    Branch(String),
    Delegate,
    Skip(String)
}

/// The local facts a bare push is judged on: the branch and the config
/// that steers `git push` without arguments. Gathered through the runner
/// seam; never from the network.
struct PushEvidence
{
    branch: String,
    upstream_remote: Option<String>,
    upstream_merge: Option<String>,
    push_remote: Option<String>,
    remote_push_default: Option<String>,
    push_default: Option<String>
}

impl PushEvidence
{
    /// The remote a bare `git push` targets, in git's precedence order.
    fn target_remote(&self) -> &str
    {
        self.push_remote
            .as_deref()
            .or(self.remote_push_default.as_deref())
            .or(self.upstream_remote.as_deref())
            .unwrap_or("origin")
    }

    /// The destination branch on the target remote, mirroring `git push`
    /// for the common `push.default` values. Configuration the resolver
    /// does not understand skips: an unproven push is withheld, not
    /// guessed at. Setups git itself refuses without touching the network
    /// (mismatched `simple` upstream, missing or triangular `upstream`)
    /// delegate, so git reports them in its own words.
    fn destination(&self) -> Destination
    {
        let branch = &self.branch;
        let same_name = format!("refs/heads/{branch}");
        let upstream_is_target =
            self.upstream_remote.as_deref() == Some(self.target_remote());

        match self.push_default.as_deref().unwrap_or("simple")
        {
            "current" => Destination::Branch(branch.clone()),
            "simple" => match &self.upstream_merge
            {
                Some(merge) if upstream_is_target && *merge != same_name =>
                    Destination::Delegate,
                _ => Destination::Branch(branch.clone())
            },
            "upstream" | "tracking" => match &self.upstream_merge
            {
                Some(merge) if upstream_is_target => match merge
                    .strip_prefix("refs/heads/")
                {
                    Some(destination) =>
                        Destination::Branch(destination.to_owned()),
                    None => Destination::Skip(format!(
                        "skipping push: upstream '{merge}' is not a \
                             branch (use 'git vmr foreach' to push anyway)"
                    ))
                },
                _ => Destination::Delegate
            },
            other => Destination::Skip(format!(
                "skipping push: cannot classify usefulness under \
                 push.default '{other}' (use 'git vmr foreach' to push anyway)"
            ))
        }
    }
}

impl Git
{
    /// Classifies a bare push from local evidence alone: delegate when the
    /// destination already exists on the target remote or the tip has
    /// novel commits, skip when the push would only create an empty
    /// branch, and err toward skipping when config resolution cannot
    /// prove usefulness.
    fn classify_bare_push(&self, repo_path: &Path) -> Result<BarePush>
    {
        let branch = match self.head(repo_path)?
        {
            Head::Branch(branch) => branch,
            // Detached and unborn heads delegate; git's repo-suffixed
            // error is acceptable there.
            Head::Detached(_) | Head::Unborn(_) =>
                return Ok(BarePush::Delegate),
        };

        let evidence = self.push_evidence(repo_path, branch)?;
        let remote = evidence.target_remote();

        let destination = match evidence.destination()
        {
            Destination::Branch(destination) => destination,
            Destination::Delegate => return Ok(BarePush::Delegate),
            Destination::Skip(reason) => return Ok(BarePush::Skip(reason))
        };

        if self.remote_tracking_ref_exists(repo_path, remote, &destination)?
        {
            // The destination exists on the target remote: an update or
            // an up-to-date no-op, never an empty branch creation.
            return Ok(BarePush::Delegate);
        }

        if self.has_novel_commits(repo_path, &evidence.branch, remote)?
        {
            return Ok(BarePush::Delegate);
        }

        Ok(BarePush::Skip(format!(
            "skipping '{}': push would only create an empty branch on '{}' \
             (use 'git vmr foreach' to push anyway)",
            evidence.branch, remote
        )))
    }

    fn push_evidence(
        &self,
        repo_path: &Path,
        branch: String
    ) -> Result<PushEvidence>
    {
        Ok(PushEvidence {
            upstream_remote: self
                .config_get(repo_path, &format!("branch.{branch}.remote"))?,
            upstream_merge: self
                .config_get(repo_path, &format!("branch.{branch}.merge"))?,
            push_remote: self.config_get(
                repo_path,
                &format!("branch.{branch}.pushRemote")
            )?,
            remote_push_default: self
                .config_get(repo_path, "remote.pushDefault")?,
            push_default: self.config_get(repo_path, "push.default")?,
            branch
        })
    }

    fn config_get(&self, repo_path: &Path, key: &str)
    -> Result<Option<String>>
    {
        let output = self.output(repo_path, ["config", "--get", key])?;

        match output.status.code()
        {
            Some(0) => Ok(Some(
                String::from_utf8_lossy(&output.stdout).trim().to_owned()
            )),
            Some(1) => Ok(None),
            _ => bail!(
                "fatal: failed to read config '{}' in '{}': {}",
                key,
                repo_path.display(),
                stderr(&output)
            )
        }
    }

    fn remote_tracking_ref_exists(
        &self,
        repo_path: &Path,
        remote: &str,
        destination: &str
    ) -> Result<bool>
    {
        let ref_name = format!("refs/remotes/{remote}/{destination}");
        let output = self.output(repo_path, [
            "show-ref", "--verify", "--quiet", &ref_name
        ])?;

        match output.status.code()
        {
            Some(0) => Ok(true),
            Some(1) => Ok(false),
            _ => bail!(
                "fatal: failed to check '{}' in '{}': {}",
                ref_name,
                repo_path.display(),
                stderr(&output)
            )
        }
    }

    /// Whether the branch tip carries commits not reachable from any
    /// remote-tracking ref of the target remote. A stale local picture
    /// answers yes and errs toward pushing, which git absorbs.
    fn has_novel_commits(
        &self,
        repo_path: &Path,
        branch: &str,
        remote: &str
    ) -> Result<bool>
    {
        let stdout = self.stdout(repo_path, [
            "rev-list",
            "-1",
            &format!("refs/heads/{branch}"),
            "--not",
            &format!("--remotes={remote}")
        ])?;

        Ok(!String::from_utf8_lossy(&stdout).trim().is_empty())
    }

    fn push(&self, repo: &Repo, push: &PushArgs) -> GitCommandResult
    {
        let mut args = vec![OsString::from("push")];

        if let Some(repository) = &push.repository
        {
            args.push(OsString::from(repository));
        }

        args.extend(push.refspecs.iter().map(OsString::from));

        let output = self.output(&repo.path, args)?;

        command_result(
            repo,
            &output,
            SuccessReport::line(Streams::StderrThenStdout, OnEmpty::Quiet),
            FailureReport::line(Streams::StderrThenStdout, "git push failed")
        )
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::git::{RepoOutcome, ScriptedFake};
    use crate::test_support::repo;

    #[test]
    fn push_success_reports_stderr_before_stdout()
    {
        // Arrange
        let git = Git::with(ScriptedFake::new().on(
            ["push"],
            0,
            "stdout chatter\n",
            "main -> main\n"
        ));

        // Act
        let outcome = git
            .push(&repo("backend", "/vmr/backend"), &PushArgs {
                repository: None,
                refspecs: vec![]
            })
            .unwrap();

        // Assert
        let RepoOutcome::Success(Some(message)) = outcome
        else
        {
            panic!("expected success with message");
        };
        assert_eq!(message.message, "main -> main");
    }

    fn evidence(branch: &str) -> PushEvidence
    {
        PushEvidence {
            branch: branch.to_owned(),
            upstream_remote: None,
            upstream_merge: None,
            push_remote: None,
            remote_push_default: None,
            push_default: None
        }
    }

    #[test]
    fn target_remote_follows_git_precedence()
    {
        let mut facts = evidence("feature");
        assert_eq!(facts.target_remote(), "origin");

        facts.upstream_remote = Some("upstream".to_owned());
        assert_eq!(facts.target_remote(), "upstream");

        facts.remote_push_default = Some("fork".to_owned());
        assert_eq!(facts.target_remote(), "fork");

        facts.push_remote = Some("mirror".to_owned());
        assert_eq!(facts.target_remote(), "mirror");
    }

    #[test]
    fn destination_defaults_to_the_branch_name_under_simple()
    {
        let facts = evidence("feature");

        let Destination::Branch(destination) = facts.destination()
        else
        {
            panic!("expected a branch destination");
        };
        assert_eq!(destination, "feature");
    }

    #[test]
    fn destination_under_simple_delegates_a_mismatched_upstream_name()
    {
        let mut facts = evidence("feature");
        facts.upstream_remote = Some("origin".to_owned());
        facts.upstream_merge = Some("refs/heads/other".to_owned());

        assert!(matches!(facts.destination(), Destination::Delegate));
    }

    #[test]
    fn destination_under_simple_ignores_an_upstream_on_another_remote()
    {
        let mut facts = evidence("feature");
        facts.upstream_remote = Some("upstream".to_owned());
        facts.upstream_merge = Some("refs/heads/other".to_owned());
        facts.push_remote = Some("origin".to_owned());

        let Destination::Branch(destination) = facts.destination()
        else
        {
            panic!("expected a branch destination");
        };
        assert_eq!(destination, "feature");
    }

    #[test]
    fn destination_under_current_is_the_branch_name()
    {
        let mut facts = evidence("feature");
        facts.push_default = Some("current".to_owned());
        facts.upstream_remote = Some("origin".to_owned());
        facts.upstream_merge = Some("refs/heads/other".to_owned());

        let Destination::Branch(destination) = facts.destination()
        else
        {
            panic!("expected a branch destination");
        };
        assert_eq!(destination, "feature");
    }

    #[test]
    fn destination_under_upstream_follows_the_merge_ref()
    {
        let mut facts = evidence("feature");
        facts.push_default = Some("upstream".to_owned());
        facts.upstream_remote = Some("origin".to_owned());
        facts.upstream_merge = Some("refs/heads/other".to_owned());

        let Destination::Branch(destination) = facts.destination()
        else
        {
            panic!("expected a branch destination");
        };
        assert_eq!(destination, "other");
    }

    #[test]
    fn destination_under_upstream_delegates_when_no_upstream_matches()
    {
        let mut facts = evidence("feature");
        facts.push_default = Some("upstream".to_owned());

        assert!(matches!(facts.destination(), Destination::Delegate));
    }

    #[test]
    fn destination_under_upstream_skips_a_non_branch_merge_ref()
    {
        let mut facts = evidence("feature");
        facts.push_default = Some("upstream".to_owned());
        facts.upstream_remote = Some("origin".to_owned());
        facts.upstream_merge = Some("refs/notes/commits".to_owned());

        let Destination::Skip(reason) = facts.destination()
        else
        {
            panic!("expected a skip");
        };
        assert!(reason.contains("refs/notes/commits"));
    }

    #[test]
    fn destination_skips_unrecognized_push_default_values()
    {
        for value in ["matching", "nothing", "surprise"]
        {
            let mut facts = evidence("feature");
            facts.push_default = Some(value.to_owned());

            let Destination::Skip(reason) = facts.destination()
            else
            {
                panic!("expected a skip for push.default '{value}'");
            };
            assert!(reason.contains(value));
        }
    }

    /// Scripts the fixed evidence-gathering sequence for a repo on branch
    /// `feature` with no push-related config set.
    fn unconfigured_feature_branch() -> ScriptedFake
    {
        ScriptedFake::new()
            .on(
                ["symbolic-ref", "--quiet", "--short", "HEAD"],
                0,
                "feature\n",
                ""
            )
            .on(
                ["show-ref", "--verify", "--quiet", "refs/heads/feature"],
                0,
                "",
                ""
            )
            .on(["config", "--get", "branch.feature.remote"], 1, "", "")
            .on(["config", "--get", "branch.feature.merge"], 1, "", "")
            .on(["config", "--get", "branch.feature.pushRemote"], 1, "", "")
            .on(["config", "--get", "remote.pushDefault"], 1, "", "")
            .on(["config", "--get", "push.default"], 1, "", "")
    }

    fn classify(fake: ScriptedFake) -> (BarePush, Vec<Vec<OsString>>)
    {
        let fake = std::sync::Arc::new(fake);
        let git = Git::with(fake.clone());
        let classification =
            git.classify_bare_push(Path::new("/vmr/backend")).unwrap();
        let calls =
            fake.calls().into_iter().map(|call| call.args).collect::<Vec<_>>();
        (classification, calls)
    }

    fn ran_rev_list(calls: &[Vec<OsString>]) -> bool
    {
        calls.iter().any(|args| args.first() == Some(&"rev-list".into()))
    }

    #[test]
    fn bare_push_delegates_when_the_destination_exists_on_the_remote()
    {
        // Arrange
        let fake = unconfigured_feature_branch().on(
            ["show-ref", "--verify", "--quiet", "refs/remotes/origin/feature"],
            0,
            "",
            ""
        );

        // Act
        let (classification, calls) = classify(fake);

        // Assert
        assert!(matches!(classification, BarePush::Delegate));
        assert!(!ran_rev_list(&calls));
    }

    #[test]
    fn bare_push_delegates_when_the_tip_has_novel_commits()
    {
        // Arrange
        let fake = unconfigured_feature_branch()
            .on(
                [
                    "show-ref",
                    "--verify",
                    "--quiet",
                    "refs/remotes/origin/feature"
                ],
                1,
                "",
                ""
            )
            .on(
                [
                    "rev-list",
                    "-1",
                    "refs/heads/feature",
                    "--not",
                    "--remotes=origin"
                ],
                0,
                "abc123\n",
                ""
            );

        // Act
        let (classification, _) = classify(fake);

        // Assert
        assert!(matches!(classification, BarePush::Delegate));
    }

    #[test]
    fn bare_push_skips_an_empty_branch_creation()
    {
        // Arrange
        let fake = unconfigured_feature_branch()
            .on(
                [
                    "show-ref",
                    "--verify",
                    "--quiet",
                    "refs/remotes/origin/feature"
                ],
                1,
                "",
                ""
            )
            .on(
                [
                    "rev-list",
                    "-1",
                    "refs/heads/feature",
                    "--not",
                    "--remotes=origin"
                ],
                0,
                "",
                ""
            );

        // Act
        let (classification, calls) = classify(fake);

        // Assert
        let BarePush::Skip(reason) = classification
        else
        {
            panic!("expected a skip");
        };
        assert!(reason.contains("empty branch"));
        assert!(reason.contains("'feature'"));
        assert!(reason.contains("'origin'"));
        assert!(!calls.iter().any(|args| args.first() == Some(&"push".into())));
    }

    #[test]
    fn bare_push_skips_a_branch_deleted_on_the_remote()
    {
        // The merged-and-deleted case: upstream config lingers, the
        // remote-tracking ref for the destination is gone, and the tip is
        // reachable from the remote's other refs.
        let fake = ScriptedFake::new()
            .on(
                ["symbolic-ref", "--quiet", "--short", "HEAD"],
                0,
                "feature\n",
                ""
            )
            .on(
                ["show-ref", "--verify", "--quiet", "refs/heads/feature"],
                0,
                "",
                ""
            )
            .on(["config", "--get", "branch.feature.remote"], 0, "origin\n", "")
            .on(
                ["config", "--get", "branch.feature.merge"],
                0,
                "refs/heads/feature\n",
                ""
            )
            .on(["config", "--get", "branch.feature.pushRemote"], 1, "", "")
            .on(["config", "--get", "remote.pushDefault"], 1, "", "")
            .on(["config", "--get", "push.default"], 1, "", "")
            .on(
                [
                    "show-ref",
                    "--verify",
                    "--quiet",
                    "refs/remotes/origin/feature"
                ],
                1,
                "",
                ""
            )
            .on(
                [
                    "rev-list",
                    "-1",
                    "refs/heads/feature",
                    "--not",
                    "--remotes=origin"
                ],
                0,
                "",
                ""
            );

        // Act
        let (classification, _) = classify(fake);

        // Assert
        assert!(matches!(classification, BarePush::Skip(_)));
    }

    #[test]
    fn bare_push_skips_unrecognized_push_default_without_ref_queries()
    {
        // Arrange
        let fake = ScriptedFake::new()
            .on(
                ["symbolic-ref", "--quiet", "--short", "HEAD"],
                0,
                "feature\n",
                ""
            )
            .on(
                ["show-ref", "--verify", "--quiet", "refs/heads/feature"],
                0,
                "",
                ""
            )
            .on(["config", "--get", "branch.feature.remote"], 1, "", "")
            .on(["config", "--get", "branch.feature.merge"], 1, "", "")
            .on(["config", "--get", "branch.feature.pushRemote"], 1, "", "")
            .on(["config", "--get", "remote.pushDefault"], 1, "", "")
            .on(["config", "--get", "push.default"], 0, "matching\n", "");

        // Act
        let (classification, calls) = classify(fake);

        // Assert
        assert!(matches!(classification, BarePush::Skip(_)));
        assert!(!ran_rev_list(&calls));
    }

    #[test]
    fn bare_push_delegates_a_detached_head()
    {
        // Arrange
        let fake = ScriptedFake::new()
            .on(["symbolic-ref", "--quiet", "--short", "HEAD"], 1, "", "")
            .on(["rev-parse", "--short=8", "HEAD"], 0, "abc12345\n", "");

        // Act
        let (classification, calls) = classify(fake);

        // Assert
        assert!(matches!(classification, BarePush::Delegate));
        assert!(
            !calls.iter().any(|args| args.first() == Some(&"config".into()))
        );
    }
}
