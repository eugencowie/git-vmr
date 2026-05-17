use crate::git::git_stdout;
use crate::vmr::Repo;
use anyhow::{Context, Result};

pub fn tags(repo: &Repo) -> Result<Option<(String, Vec<String>)>>
{
    let tags_output = git_stdout(&repo.path, [
        "for-each-ref",
        "--format=%(refname:short)",
        "refs/tags"
    ])
    .with_context(|| {
        format!("failed to read tag information for '{}'", repo.path.display())
    })?;
    let tags = String::from_utf8_lossy(&tags_output)
        .lines()
        .map(str::to_owned)
        .collect::<Vec<_>>();

    Ok(Some((repo.name.clone(), tags)))
}
