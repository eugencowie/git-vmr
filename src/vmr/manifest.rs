use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::path::{Component, Path};
use toml::Spanned;
use toml_edit::{DocumentMut, Item, Table, value};

/// The `.gitvmr` manifest: the child repos the root repo declares.
#[derive(Debug, Default, Eq, PartialEq)]
pub struct Manifest
{
    declarations: Vec<Declaration>
}

/// One child repo as the manifest declares it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Declaration
{
    pub name: String,
    pub path: RepoPath,
    pub url: RepoUrl
}

impl Manifest
{
    pub fn parse(text: &str) -> Result<Manifest>
    {
        // Parse TOML
        let raw: RawManifest = toml::from_str(text)?;

        // Check for duplicate paths in file order, so the later declaration
        // is the one reported
        let mut entries = raw.repo.into_iter().collect::<Vec<_>>();
        entries.sort_by_key(|(_, raw)| raw.path.span().start);
        let mut declared_by = HashMap::new();
        for (name, raw) in &entries
        {
            let path = raw.path.get_ref();
            if let Some(first) = declared_by.insert(path, name)
            {
                bail!(
                    "line {}: path '{path}' is already declared by '{first}'",
                    line_of(text, raw.path.span().start)
                );
            }
        }

        // Build declarations sorted by path
        let mut declarations = entries
            .into_iter()
            .map(|(name, raw)| Declaration {
                name,
                path: raw.path.into_inner(),
                url: raw.url
            })
            .collect::<Vec<_>>();
        declarations.sort_by(|a, b| a.path.cmp(&b.path));

        Ok(Manifest { declarations })
    }

    /// The declarations, sorted by path.
    pub fn declarations(&self) -> &[Declaration]
    {
        &self.declarations
    }
}

/// The 1-based line of a byte offset in `text`.
fn line_of(text: &str, offset: usize) -> usize
{
    text[..offset].matches('\n').count() + 1
}

/// Manifest text under edit. Edits go through `toml_edit` rather than
/// re-serializing a parsed manifest, so the comments and formatting of a
/// hand-edited, committed file survive.
pub struct ManifestEditor
{
    document: DocumentMut
}

impl ManifestEditor
{
    /// An empty manifest.
    pub fn new() -> Self
    {
        // An implicit `[repo]` table writes no header, so an empty manifest
        // is an empty file
        let mut document = DocumentMut::new();
        let mut repos = Table::new();
        repos.set_implicit(true);
        document.insert("repo", Item::Table(repos));
        ManifestEditor { document }
    }

    /// Appends a `[repo.<name>]` table declaring a child repo.
    pub fn declare(&mut self, declaration: &Declaration)
    {
        // Build table, after a blank line unless it is the first
        let mut table = Table::new();
        table.insert("path", value(declaration.path.as_str()));
        table.insert("url", value(declaration.url.as_str()));
        if !self.is_empty()
        {
            table.decor_mut().set_prefix("\n");
        }

        // Add it under `[repo]`
        self.document["repo"]
            .as_table_mut()
            .expect("the repo table is created with the editor")
            .insert(&declaration.name, Item::Table(table));
    }

    /// Whether the manifest declares no child repos.
    pub fn is_empty(&self) -> bool
    {
        self.document["repo"].as_table().is_none_or(Table::is_empty)
    }
}

impl fmt::Display for ManifestEditor
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(f, "{}", self.document)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawManifest
{
    #[serde(default)]
    repo: BTreeMap<String, RawDeclaration>
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawDeclaration
{
    path: Spanned<RepoPath>,
    url: RepoUrl
}

/// A child repo path: a single directory name directly under the VMR
/// root, never one the root repo or git-vmr reserves.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(try_from = "String")]
pub struct RepoPath(String);

impl RepoPath
{
    pub fn parse(path: String) -> Result<RepoPath, String>
    {
        // Check for one normal component, with no separator of either kind
        let mut components = Path::new(&path).components();
        let single_name = matches!(
            (components.next(), components.next()),
            (Some(Component::Normal(_)), None)
        ) && !path.contains(['/', '\\']);

        // Reject anything else, and the reserved names
        if !single_name || path == ".git" || path == ".gitvmr"
        {
            return Err(format!(
                "path '{path}' must be a single directory name, other than \
                 '.git' or '.gitvmr'"
            ));
        }

        Ok(RepoPath(path))
    }

    pub fn as_str(&self) -> &str
    {
        &self.0
    }
}

impl TryFrom<String> for RepoPath
{
    type Error = String;

    fn try_from(path: String) -> Result<Self, Self::Error>
    {
        RepoPath::parse(path)
    }
}

impl fmt::Display for RepoPath
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        f.write_str(&self.0)
    }
}

/// A child repo's remote URL, absolute or relative to the root repo's.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(try_from = "String")]
pub struct RepoUrl(String);

impl RepoUrl
{
    pub fn parse(url: String) -> Result<RepoUrl, String>
    {
        if url.is_empty()
        {
            return Err("url must not be empty".to_owned());
        }

        Ok(RepoUrl(url))
    }

    pub fn as_str(&self) -> &str
    {
        &self.0
    }

    /// Whether the URL is relative to the root repo's, as a URL starting
    /// with `./` or `../` is.
    pub fn is_relative(&self) -> bool
    {
        starts_relative(&self.0)
    }

    /// The URL to clone from: an absolute URL as declared, or a relative one
    /// resolved against `root_url`, the root repo's `origin` URL, the way
    /// git resolves a relative submodule URL.
    pub fn resolve(&self, root_url: Option<&str>) -> Result<String>
    {
        // Keep an absolute URL
        if !self.is_relative()
        {
            return Ok(self.0.clone());
        }

        // Require the root repo's URL, without a trailing separator
        let Some(root_url) = root_url
        else
        {
            bail!(
                "cannot resolve relative URL '{}': the root repo has no \
                 'origin' remote",
                self.0
            );
        };
        let root_url = root_url.strip_suffix(is_dir_sep).unwrap_or(root_url);

        // Start a relative root URL with `./`, so climbing out of it stays
        // relative
        let root_is_relative = is_local(root_url) && !is_absolute(root_url);
        let mut base = if root_is_relative && !starts_relative(root_url)
        {
            format!("./{root_url}")
        }
        else
        {
            root_url.to_owned()
        };

        // Drop one component of the base per leading `../`, skipping `./`
        let mut url = self.0.as_str();
        let mut colon_separated = false;
        loop
        {
            if let Some(rest) = strip_dots(url, "..")
            {
                url = rest;
                colon_separated |= chop_last_component(
                    &mut base,
                    root_is_relative
                )
                .with_context(|| {
                    format!(
                        "cannot resolve relative URL '{}' against '{root_url}'",
                        self.0
                    )
                })?;
            }
            else if let Some(rest) = strip_dots(url, ".")
            {
                url = rest;
            }
            else
            {
                break;
            }
        }

        // Join the rest onto the base, without the rest's trailing slash or
        // a leading `./`
        let separator = if colon_separated { ':' } else { '/' };
        let joined = format!("{base}{separator}{url}");
        let joined = if url.ends_with('/')
        {
            &joined[..joined.len() - 1]
        }
        else
        {
            &joined
        };
        Ok(joined.strip_prefix("./").unwrap_or(joined).to_owned())
    }
}

/// Whether `url` starts with `./` or `../`.
fn starts_relative(url: &str) -> bool
{
    strip_dots(url, ".").is_some() || strip_dots(url, "..").is_some()
}

/// `url` without a leading `dots` and directory separator, if it has them.
fn strip_dots<'u>(url: &'u str, dots: &str) -> Option<&'u str>
{
    url.strip_prefix(dots)?.strip_prefix(is_dir_sep)
}

/// Removes the last component of `base`, returning whether a colon rather
/// than a separator delimited it, as in scp-style `host:repo`. `None` when
/// there is nothing left to remove.
fn chop_last_component(base: &mut String, is_relative: bool) -> Option<bool>
{
    // Cut at the last separator, then at the last colon
    if let Some(index) = base.rfind(is_dir_sep)
    {
        base.truncate(index);
        return Some(false);
    }
    if let Some(index) = base.rfind(':')
    {
        base.truncate(index);
        return Some(true);
    }

    // A bare absolute name climbs to the current directory, once
    if is_relative || base == "."
    {
        return None;
    }
    ".".clone_into(base);
    Some(false)
}

/// Whether `url` is a local path rather than a URL or scp-style address:
/// no colon, or a separator before the first one.
fn is_local(url: &str) -> bool
{
    match url.find(':')
    {
        None => true,
        Some(colon) =>
            url.find(is_dir_sep).is_some_and(|slash| slash < colon)
                || has_drive_prefix(url),
    }
}

fn is_absolute(path: &str) -> bool
{
    path.starts_with(is_dir_sep) || has_drive_prefix(path)
}

/// Whether `path` starts with a Windows drive letter, as in `C:`.
fn has_drive_prefix(path: &str) -> bool
{
    let mut chars = path.chars();
    cfg!(windows)
        && chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.next() == Some(':')
}

/// Git's native directory separators.
fn is_dir_sep(c: char) -> bool
{
    c == '/' || (cfg!(windows) && c == '\\')
}

impl TryFrom<String> for RepoUrl
{
    type Error = String;

    fn try_from(url: String) -> Result<Self, Self::Error>
    {
        RepoUrl::parse(url)
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn declaration(name: &str, path: &str, url: &str) -> Declaration
    {
        Declaration {
            name: name.to_owned(),
            path: RepoPath::parse(path.to_owned()).unwrap(),
            url: RepoUrl::parse(url.to_owned()).unwrap()
        }
    }

    #[test]
    fn parses_declarations_sorted_by_path()
    {
        // Act
        let manifest = Manifest::parse(
            "[repo.zeta]\npath = \"a\"\nurl = \"git://host/zeta.git\"\n\n\
             [repo.alpha]\npath = \"b\"\nurl = \"../alpha.git\"\n"
        )
        .unwrap();

        // Assert
        assert_eq!(manifest.declarations(), [
            declaration("zeta", "a", "git://host/zeta.git"),
            declaration("alpha", "b", "../alpha.git")
        ]);
    }

    #[test]
    fn rejects_a_path_that_is_not_a_single_directory_name()
    {
        for path in ["a/b", "", ".", "..", ".git", ".gitvmr", "/abs", "a\\b"]
        {
            // Act
            let err = Manifest::parse(&format!(
                "[repo.x]\nurl = \"u\"\npath = '{path}'\n"
            ))
            .unwrap_err();

            // Assert
            let message = format!("{err:#}");
            assert!(message.contains("line 3"), "{path}: {message}");
            assert!(
                message.contains("must be a single directory name"),
                "{path}: {message}"
            );
        }
    }

    #[test]
    fn rejects_an_empty_url()
    {
        // Act
        let err = Manifest::parse("[repo.x]\npath = \"x\"\nurl = \"\"\n")
            .unwrap_err();

        // Assert
        let message = format!("{err:#}");
        assert!(message.contains("line 3"), "{message}");
        assert!(message.contains("url must not be empty"), "{message}");
    }

    #[test]
    fn resolves_a_relative_url_against_the_root_repos_url()
    {
        for (root_url, url, resolved) in [
            (
                "https://host/org/root.git",
                "../lib.git",
                "https://host/org/lib.git"
            ),
            (
                "https://host/org/root.git/",
                "../lib.git",
                "https://host/org/lib.git"
            ),
            (
                "https://host/org/root.git",
                "../../lib.git",
                "https://host/lib.git"
            ),
            (
                "https://host/org/root.git",
                "./lib.git",
                "https://host/org/root.git/lib.git"
            ),
            ("https://host/org/root.git", "../lib/", "https://host/org/lib"),
            ("https://host/org/root.git", "../", "https://host/org/"),
            ("git@host:org/root.git", "../lib.git", "git@host:org/lib.git"),
            ("git@host:root.git", "../lib.git", "git@host:lib.git"),
            ("/srv/git/root", "../lib", "/srv/git/lib"),
            ("../up/root", "../lib", "../up/lib"),
            ("up/root", "./lib", "up/root/lib"),
            ("root", "../lib", "lib")
        ]
        {
            // Arrange
            let url = RepoUrl::parse(url.to_owned()).unwrap();

            // Act
            let result = url.resolve(Some(root_url)).unwrap();

            // Assert
            assert_eq!(result, resolved, "{root_url} + {url:?}");
        }
    }

    #[test]
    fn keeps_an_absolute_url_as_declared()
    {
        for url in ["git@other:lib.git", "https://host/lib.git", "/srv/lib"]
        {
            // Arrange
            let url = RepoUrl::parse(url.to_owned()).unwrap();

            // Act
            let result = url.resolve(None).unwrap();

            // Assert
            assert_eq!(result, url.as_str());
        }
    }

    #[test]
    fn fails_to_resolve_a_relative_url_without_a_root_repo_url()
    {
        // Arrange
        let url = RepoUrl::parse("../lib.git".to_owned()).unwrap();

        // Act
        let err = url.resolve(None).unwrap_err();

        // Assert
        assert_eq!(
            format!("{err:#}"),
            "cannot resolve relative URL '../lib.git': the root repo has no \
             'origin' remote"
        );
    }

    #[test]
    fn fails_to_resolve_a_relative_url_that_climbs_too_far()
    {
        // Arrange
        let url = RepoUrl::parse("../../lib".to_owned()).unwrap();

        // Act
        let err = url.resolve(Some("root")).unwrap_err();

        // Assert
        assert_eq!(
            format!("{err:#}"),
            "cannot resolve relative URL '../../lib' against 'root'"
        );
    }

    #[test]
    fn rejects_unknown_keys()
    {
        // Act
        let err = Manifest::parse("[repo.x]\npath = \"x\"\nulr = \"u\"\n")
            .unwrap_err();

        // Assert
        assert!(format!("{err:#}").contains("unknown field `ulr`"));
    }

    #[test]
    fn rejects_duplicate_paths_at_the_later_declaration()
    {
        // Act
        let err = Manifest::parse(
            "[repo.a]\npath = \"x\"\nurl = \"u\"\n\n\
             [repo.b]\npath = \"x\"\nurl = \"v\"\n"
        )
        .unwrap_err();

        // Assert
        assert_eq!(
            format!("{err:#}"),
            "line 6: path 'x' is already declared by 'a'"
        );
    }

    #[test]
    fn editor_writes_declarations_that_parse_back()
    {
        // Arrange
        let mut editor = ManifestEditor::new();

        // Act
        editor.declare(&declaration("lib.foo", "foo", "../foo.git"));
        editor.declare(&declaration("bar", "bar", "git@host:bar.git"));

        // Assert
        let text = editor.to_string();
        assert_eq!(
            text,
            "[repo.\"lib.foo\"]\npath = \"foo\"\nurl = \"../foo.git\"\n\n\
             [repo.bar]\npath = \"bar\"\nurl = \"git@host:bar.git\"\n"
        );
        assert_eq!(Manifest::parse(&text).unwrap().declarations(), [
            declaration("bar", "bar", "git@host:bar.git"),
            declaration("lib.foo", "foo", "../foo.git")
        ]);
    }
}
