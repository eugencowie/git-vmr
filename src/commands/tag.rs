use crate::workspace::Workspace;
use anstyle::{AnsiColor, Style};
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};

pub fn create(workspace: &Workspace, tag_name: &str) -> Result<()>
{
    // Create tag in each child repository
    workspace.run(|git, repo| git.tag(&repo.name, &repo.path, tag_name))
}

pub fn delete(workspace: &Workspace, tag_name: &str) -> Result<()>
{
    // Delete tag in each child repository
    workspace.run(|git, repo| git.delete_tag(&repo.name, &repo.path, tag_name))
}

pub fn tag(workspace: &Workspace) -> Result<()>
{
    // Collect tag information from child repositories, in repo order
    let tags = workspace
        .map(|git, repo| git.tags(repo))?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    // Print results
    anstream::print!("{}", render_tags(&tags));
    Ok(())
}

fn render_tags(repos: &[(String, Vec<String>)]) -> String
{
    let mut output = String::new();
    let repo_list_style =
        Style::new().fg_color(Some(AnsiColor::BrightBlack.into()));
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
            output.push(' ');
            output.push_str(&format!(
                "{}({}){}",
                repo_list_style.render(),
                repo_names.join(", "),
                repo_list_style.render_reset()
            ));
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
        assert!(output.contains("v1.1.0 \x1b[90m(frontend)\x1b[0m\n"));
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

        assert_eq!(output, "v1.2.0 \x1b[90m(backend, frontend)\x1b[0m\n");
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
            concat!(
                "v1.0.0 \x1b[90m(backend)\x1b[0m\n",
                "v1.1.0 \x1b[90m(frontend)\x1b[0m\n",
                "v2.0.0 \x1b[90m(backend)\x1b[0m\n"
            )
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
