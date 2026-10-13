use anyhow::{Result, bail};
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
        let mut document = DocumentMut::new();
        let mut repos = Table::new();
        repos.set_implicit(true);
        document.insert("repo", Item::Table(repos));
        ManifestEditor { document }
    }

    /// Appends a `[repo.<name>]` table declaring a child repo.
    pub fn declare(&mut self, declaration: &Declaration)
    {
        let mut table = Table::new();
        table.insert("path", value(declaration.path.as_str()));
        table.insert("url", value(declaration.url.as_str()));
        if !self.is_empty()
        {
            table.decor_mut().set_prefix("\n");
        }

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
        let mut components = Path::new(&path).components();
        let single_name = matches!(
            (components.next(), components.next()),
            (Some(Component::Normal(_)), None)
        ) && !path.contains(['/', '\\']);

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
