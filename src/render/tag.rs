use crate::render::{SuffixPolicy, repo_list_suffix};
use std::collections::{BTreeMap, BTreeSet};

/// Renders the tag list grouped across child repos, in tag name order.
pub fn tags(repos: &[(String, Vec<String>)]) -> String
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

        let repo_names = tag_repos.into_iter().collect::<Vec<_>>();
        output.push_str(&repo_list_suffix(
            &repo_names,
            repo_count,
            SuffixPolicy::Truncated
        ));

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

        let output = tags(&repos);

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

        let output = tags(&repos);

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

        let output = tags(&repos);

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

        let output = tags(&repos);

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
        assert_eq!(tags(&[]), "");

        let repos = vec![("backend".to_owned(), vec![])];
        assert_eq!(tags(&repos), "");
    }
}
