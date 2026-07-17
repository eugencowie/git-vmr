use crate::cli::CliContext;
use crate::render::{self, Rendered};
use crate::workspace::Workspace;
use anyhow::Result;

pub fn run(
    workspace: &Workspace,
    _context: &CliContext,
    args: &PushArgs
) -> Result<Rendered>
{
    // Push, in each child repository, only the useful subset of the push:
    // a repo may report per-refspec skips alongside its push result.
    let results =
        workspace.map(|git, repo| Ok(git.classified_push(repo, args)))?;

    render::outcomes(
        results.into_iter().flatten().collect(),
        workspace.repos().iter().map(|repo| repo.name.as_str())
    )
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

/// How one explicit refspec classified in one repo: pass it to `git push`
/// or withhold it, with the reason to report.
enum RefspecClass
{
    Push,
    Skip(String)
}

/// One repo's classified push: the refspec subset to attempt (`None`
/// means no git invocation at all) and the skip reasons to report
/// alongside it.
struct PushPlan
{
    attempt: Option<Vec<String>>,
    skips: Vec<String>
}

impl PushPlan
{
    fn attempt_all(refspecs: Vec<String>) -> Self
    {
        Self { attempt: Some(refspecs), skips: Vec::new() }
    }

    fn from_bare(classification: BarePush) -> Self
    {
        match classification
        {
            BarePush::Delegate => Self::attempt_all(Vec::new()),
            BarePush::Skip(reason) =>
                Self { attempt: None, skips: vec![reason] },
        }
    }
}

/// A repository argument that cannot be a configured remote name — git
/// forbids '/' and ':' in remote names — is a URL or path: there is no
/// local picture of it, and typing one is the strongest push intent, so
/// the whole push delegates unfiltered.
fn is_url(repository: &str) -> bool
{
    repository.contains('/')
        || repository.contains(':')
        || repository.contains('\\')
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
    explicit_remote: Option<String>,
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
        self.explicit_remote
            .as_deref()
            .or(self.push_remote.as_deref())
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
    /// Classifies the push for one repo and runs its useful subset,
    /// returning per-refspec skips alongside the push result. A repo
    /// whose subset is empty gets no git invocation.
    fn classified_push(
        &self,
        repo: &Repo,
        args: &PushArgs
    ) -> Vec<GitCommandResult>
    {
        let plan = match self.plan_push(&repo.path, args)
        {
            Ok(plan) => plan,
            Err(error) => return vec![Err(error)]
        };

        let mut results = plan
            .skips
            .into_iter()
            .map(|reason| Ok(skip_message(&repo.name, reason)))
            .collect::<Vec<_>>();

        if let Some(refspecs) = plan.attempt
        {
            results.push(self.push(
                repo,
                args.repository.as_deref(),
                &refspecs
            ));
        }

        results
    }

    /// Classifies one repo's push into its plan. Bare push is the
    /// one-refspec degenerate case; a URL repository argument delegates
    /// the entire push unfiltered.
    fn plan_push(&self, repo_path: &Path, args: &PushArgs) -> Result<PushPlan>
    {
        let Some(repository) = &args.repository
        else
        {
            return Ok(PushPlan::from_bare(
                self.classify_bare_push(repo_path, None)?
            ));
        };

        if is_url(repository)
        {
            return Ok(PushPlan::attempt_all(args.refspecs.clone()));
        }

        if args.refspecs.is_empty()
        {
            return Ok(PushPlan::from_bare(
                self.classify_bare_push(repo_path, Some(repository))?
            ));
        }

        let mut useful = Vec::new();
        let mut skips = Vec::new();

        for refspec in &args.refspecs
        {
            match self.classify_refspec(repo_path, repository, refspec)?
            {
                RefspecClass::Push => useful.push(refspec.clone()),
                RefspecClass::Skip(reason) => skips.push(reason)
            }
        }

        Ok(PushPlan { attempt: (!useful.is_empty()).then_some(useful), skips })
    }

    /// Classifies one explicit refspec by ref class: branch refspecs get
    /// the usefulness predicate; deletes and tags push on explicit
    /// intent; wildcards skip as unclassifiable; anything git must
    /// resolve itself (unknown sources, non-branch destinations)
    /// delegates so git reports it in its own words.
    fn classify_refspec(
        &self,
        repo_path: &Path,
        remote: &str,
        refspec: &str
    ) -> Result<RefspecClass>
    {
        let spec = refspec.strip_prefix('+').unwrap_or(refspec);

        if spec.contains('*') || spec == ":"
        {
            // The bare matching refspec ':' expands like a wildcard: it
            // names no single ref this side can classify.
            let kind = if spec == ":" { "matching" } else { "wildcard" };
            return Ok(RefspecClass::Skip(format!(
                "skipping '{refspec}': cannot classify a {kind} refspec \
                 (use 'git vmr foreach' to push anyway)"
            )));
        }

        let (source, destination) = match spec.split_once(':')
        {
            // Delete refspecs always push: removing a ref never creates
            // pollution.
            Some(("", _)) => return Ok(RefspecClass::Push),
            Some((source, destination)) => (source, Some(destination)),
            None => (spec, None)
        };

        if let Some(destination) = destination
            && destination.starts_with("refs/")
            && !destination.starts_with("refs/heads/")
        {
            // An explicit non-branch destination (tags, notes) pushes on
            // intent: only branch creation constitutes pollution.
            return Ok(RefspecClass::Push);
        }

        let full_name = self.symbolic_full_name(repo_path, source)?;

        match (full_name.as_deref(), destination)
        {
            // Tag sources always push: tag existence on the remote is
            // locally unverifiable, and a tag's commit already being on
            // the remote is the normal case.
            (Some(name), _) if name.starts_with("refs/tags/") =>
                return Ok(RefspecClass::Push),
            // Without a destination, a source that is not a local branch
            // (unknown names, detached HEAD, raw ids) delegates.
            (Some(name), None) if !name.starts_with("refs/heads/") =>
                return Ok(RefspecClass::Push),
            (None, None) => return Ok(RefspecClass::Push),
            _ =>
            {}
        }

        let destination_branch = match destination
        {
            Some(destination) => destination.trim_start_matches("refs/heads/"),
            None => match full_name
                .as_deref()
                .and_then(|name| name.strip_prefix("refs/heads/"))
            {
                Some(branch) => branch,
                None => return Ok(RefspecClass::Push)
            }
        };

        let tip = full_name.as_deref().unwrap_or(source);

        if full_name.is_none() && !self.rev_resolves(repo_path, tip)?
        {
            // The source does not resolve locally: delegate so git
            // reports the bad refspec itself.
            return Ok(RefspecClass::Push);
        }

        if self.remote_tracking_ref_exists(
            repo_path,
            remote,
            destination_branch
        )? || self.has_novel_commits(repo_path, tip, remote)?
        {
            return Ok(RefspecClass::Push);
        }

        Ok(RefspecClass::Skip(format!(
            "skipping '{refspec}': push would only create an empty branch \
             on '{remote}' (use 'git vmr foreach' to push anyway)"
        )))
    }

    /// Classifies a bare push from local evidence alone: delegate when the
    /// destination already exists on the target remote or the tip has
    /// novel commits, skip when the push would only create an empty
    /// branch, and err toward skipping when config resolution cannot
    /// prove usefulness. An explicit remote argument overrides config
    /// resolution of the target remote.
    fn classify_bare_push(
        &self,
        repo_path: &Path,
        explicit_remote: Option<&str>
    ) -> Result<BarePush>
    {
        let branch = match self.head(repo_path)?
        {
            Head::Branch(branch) => branch,
            // Detached and unborn heads delegate; git's repo-suffixed
            // error is acceptable there.
            Head::Detached(_) | Head::Unborn(_) =>
                return Ok(BarePush::Delegate),
        };

        let evidence =
            self.push_evidence(repo_path, branch, explicit_remote)?;
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

        if self.has_novel_commits(
            repo_path,
            &format!("refs/heads/{}", evidence.branch),
            remote
        )?
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
        branch: String,
        explicit_remote: Option<&str>
    ) -> Result<PushEvidence>
    {
        Ok(PushEvidence {
            explicit_remote: explicit_remote.map(str::to_owned),
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

    /// Whether the tip carries commits not reachable from any
    /// remote-tracking ref of the target remote. A stale local picture
    /// answers yes and errs toward pushing, which git absorbs.
    fn has_novel_commits(
        &self,
        repo_path: &Path,
        tip: &str,
        remote: &str
    ) -> Result<bool>
    {
        let stdout = self.stdout(repo_path, [
            "rev-list",
            "-1",
            tip,
            "--not",
            &format!("--remotes={remote}")
        ])?;

        Ok(!String::from_utf8_lossy(&stdout).trim().is_empty())
    }

    /// The full ref name a revision resolves to, or `None` when it does
    /// not resolve or names no ref (raw ids resolve to nothing).
    fn symbolic_full_name(
        &self,
        repo_path: &Path,
        rev: &str
    ) -> Result<Option<String>>
    {
        let output =
            self.output(repo_path, ["rev-parse", "--symbolic-full-name", rev])?;

        if !output.status.success()
        {
            return Ok(None);
        }

        let name = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        Ok((!name.is_empty()).then_some(name))
    }

    fn rev_resolves(&self, repo_path: &Path, rev: &str) -> Result<bool>
    {
        let output =
            self.output(repo_path, ["rev-parse", "--verify", "--quiet", rev])?;
        Ok(output.status.success())
    }

    fn push(
        &self,
        repo: &Repo,
        repository: Option<&str>,
        refspecs: &[String]
    ) -> GitCommandResult
    {
        let mut args = vec![OsString::from("push")];

        if let Some(repository) = repository
        {
            args.push(OsString::from(repository));
        }

        args.extend(refspecs.iter().map(OsString::from));

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
        let outcome =
            git.push(&repo("backend", "/vmr/backend"), None, &[]).unwrap();

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
            explicit_remote: None,
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
            git.classify_bare_push(Path::new("/vmr/backend"), None).unwrap();
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

    fn classify_refspec(
        fake: ScriptedFake,
        refspec: &str
    ) -> (RefspecClass, Vec<Vec<OsString>>)
    {
        let fake = std::sync::Arc::new(fake);
        let git = Git::with(fake.clone());
        let class = git
            .classify_refspec(Path::new("/vmr/backend"), "origin", refspec)
            .unwrap();
        let calls =
            fake.calls().into_iter().map(|call| call.args).collect::<Vec<_>>();
        (class, calls)
    }

    #[test]
    fn delete_refspec_always_pushes_without_queries()
    {
        let (class, calls) = classify_refspec(ScriptedFake::new(), ":gone");

        assert!(matches!(class, RefspecClass::Push));
        assert!(calls.is_empty());
    }

    #[test]
    fn wildcard_refspec_skips_as_unclassifiable()
    {
        let (class, calls) =
            classify_refspec(ScriptedFake::new(), "refs/heads/*:refs/heads/*");

        let RefspecClass::Skip(reason) = class
        else
        {
            panic!("expected a skip");
        };
        assert!(reason.contains("refs/heads/*:refs/heads/*"));
        assert!(reason.contains("wildcard"));
        assert!(calls.is_empty());
    }

    #[test]
    fn matching_refspec_skips_as_unclassifiable()
    {
        let (class, calls) = classify_refspec(ScriptedFake::new(), ":");

        let RefspecClass::Skip(reason) = class
        else
        {
            panic!("expected a skip");
        };
        assert!(reason.contains("matching"));
        assert!(calls.is_empty());
    }

    #[test]
    fn tag_refspec_always_pushes_without_a_usefulness_check()
    {
        // Arrange
        let fake = ScriptedFake::new().on(
            ["rev-parse", "--symbolic-full-name", "v1.0.0"],
            0,
            "refs/tags/v1.0.0\n",
            ""
        );

        // Act
        let (class, calls) = classify_refspec(fake, "v1.0.0");

        // Assert
        assert!(matches!(class, RefspecClass::Push));
        assert!(!ran_rev_list(&calls));
    }

    #[test]
    fn explicit_tag_destination_pushes_without_resolving_the_source()
    {
        let (class, calls) =
            classify_refspec(ScriptedFake::new(), "v1.0.0:refs/tags/v1.0.0");

        assert!(matches!(class, RefspecClass::Push));
        assert!(calls.is_empty());
    }

    #[test]
    fn unresolvable_source_delegates_so_git_reports_it()
    {
        // Arrange
        let fake = ScriptedFake::new().on(
            ["rev-parse", "--symbolic-full-name", "nosuch"],
            128,
            "",
            ""
        );

        // Act
        let (class, _) = classify_refspec(fake, "nosuch");

        // Assert
        assert!(matches!(class, RefspecClass::Push));
    }

    #[test]
    fn branch_refspec_skips_an_empty_branch_creation()
    {
        // Arrange
        let fake = ScriptedFake::new()
            .on(
                ["rev-parse", "--symbolic-full-name", "feature"],
                0,
                "refs/heads/feature\n",
                ""
            )
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
        let (class, _) = classify_refspec(fake, "feature");

        // Assert
        let RefspecClass::Skip(reason) = class
        else
        {
            panic!("expected a skip");
        };
        assert!(reason.contains("'feature'"));
        assert!(reason.contains("empty branch"));
        assert!(reason.contains("'origin'"));
    }

    #[test]
    fn branch_refspec_pushes_when_the_tip_has_novel_commits()
    {
        // Arrange
        let fake = ScriptedFake::new()
            .on(
                ["rev-parse", "--symbolic-full-name", "feature"],
                0,
                "refs/heads/feature\n",
                ""
            )
            .on(
                [
                    "show-ref",
                    "--verify",
                    "--quiet",
                    "refs/remotes/origin/other"
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

        // Act: an explicit destination names the branch the predicate
        // checks on the remote.
        let (class, _) = classify_refspec(fake, "+feature:refs/heads/other");

        // Assert
        assert!(matches!(class, RefspecClass::Push));
    }

    #[test]
    fn branch_refspec_pushes_when_the_destination_exists_on_the_remote()
    {
        // Arrange
        let fake = ScriptedFake::new()
            .on(
                ["rev-parse", "--symbolic-full-name", "feature"],
                0,
                "refs/heads/feature\n",
                ""
            )
            .on(
                [
                    "show-ref",
                    "--verify",
                    "--quiet",
                    "refs/remotes/origin/feature"
                ],
                0,
                "",
                ""
            );

        // Act
        let (class, calls) = classify_refspec(fake, "feature");

        // Assert
        assert!(matches!(class, RefspecClass::Push));
        assert!(!ran_rev_list(&calls));
    }

    #[test]
    fn url_repository_argument_is_recognized()
    {
        assert!(is_url("/repos/project.git"));
        assert!(is_url("git@example.com:project.git"));
        assert!(is_url("https://example.com/project.git"));
        assert!(is_url(r"C:\repos\project"));
        assert!(!is_url("origin"));
        assert!(!is_url("upstream"));
    }

    #[test]
    fn explicit_remote_argument_overrides_configured_remotes()
    {
        let mut facts = evidence("feature");
        facts.push_remote = Some("mirror".to_owned());
        facts.explicit_remote = Some("origin".to_owned());

        assert_eq!(facts.target_remote(), "origin");
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
