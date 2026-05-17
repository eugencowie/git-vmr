use crate::git::{self};
use crate::vmr::Vmr;
use anyhow::Result;
use rayon::prelude::*;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub fn tag(working_dir: &Path) -> Result<()>
{
    // Find virtual monorepo
    let vmr = Vmr::find(working_dir)?;

    // Get list of repositories
    let repos = vmr.repos()?;

    // Collect tag information from repositories
    let mut tags = repos
        .par_iter()
        .filter_map(|repo| git::tags(repo).transpose())
        .collect::<Result<Vec<_>>>()?;

    // Keep repository order deterministic
    tags.sort_by(|(a, _), (b, _)| a.cmp(b));

    // Print results
    anstream::print!("{}", render_tags(&tags));
    Ok(())
}

fn render_tags(repos: &[(String, Vec<String>)]) -> String
{
    let mut output = String::new();
    let repo_count = repos.len();
    let mut tag_groups: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();

    for (repo, tags) in repos
    {
        for tag in tags
        {
            tag_groups.entry(tag.as_str()).or_default().insert(repo.as_str());
        }
    }

    for (tag, tag_repos) in tag_groups
    {
        output.push_str(tag);

        if tag_repos.len() != repo_count
        {
            let repo_names = tag_repos.into_iter().collect::<Vec<_>>();
            output.push_str(&format!(" ({})", repo_names.join(", ")));
        }

        output.push('\n');
    }

    output
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn renders_shared_tag_without_repo_list()
    {
        let repos = vec![
            ("backend".to_owned(), vec!["v1.0.0".to_owned()]),
            ("frontend".to_owned(), vec!["v1.0.0".to_owned()]),
        ];

        let output = render_tags(&repos);

        assert_eq!(output, "v1.0.0\n");
    }

    #[test]
    fn renders_partial_tag_with_repo_list()
    {
        let repos = vec![
            ("backend".to_owned(), vec!["v1.0.0".to_owned()]),
            ("frontend".to_owned(), vec![
                "v1.0.0".to_owned(),
                "v1.1.0".to_owned(),
            ]),
        ];

        let output = render_tags(&repos);

        assert!(output.contains("v1.0.0\n"));
        assert!(output.contains("v1.1.0 (frontend)\n"));
    }

    #[test]
    fn renders_partial_tag_with_sorted_repo_list()
    {
        let repos = vec![
            ("backend".to_owned(), vec!["v1.2.0".to_owned()]),
            ("frontend".to_owned(), vec!["v1.2.0".to_owned()]),
            ("tools".to_owned(), vec![]),
        ];

        let output = render_tags(&repos);

        assert_eq!(output, "v1.2.0 (backend, frontend)\n");
    }

    #[test]
    fn renders_tags_in_name_order()
    {
        let repos = vec![
            ("backend".to_owned(), vec![
                "v2.0.0".to_owned(),
                "v1.0.0".to_owned(),
            ]),
            ("frontend".to_owned(), vec!["v1.1.0".to_owned()]),
        ];

        let output = render_tags(&repos);

        assert_eq!(
            output,
            "v1.0.0 (backend)\nv1.1.0 (frontend)\nv2.0.0 (backend)\n"
        );
    }

    #[test]
    fn renders_empty_output_for_no_repos_or_tag_refs()
    {
        assert_eq!(render_tags(&[]), "");

        let repos = vec![("backend".to_owned(), vec![])];
        assert_eq!(render_tags(&repos), "");
    }
}
