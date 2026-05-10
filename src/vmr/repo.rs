use std::path::PathBuf;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Repo
{
    pub name: String,
    pub path: PathBuf
}

impl Repo
{
    pub fn new(name: String, path: PathBuf) -> Self
    {
        Self { name, path }
    }

    pub fn find(path: PathBuf) -> Option<Repo>
    {
        // Check for repository marker
        if !path.join(".git").exists()
        {
            return None;
        }

        // Use directory name as repository name
        let name = path.file_name().and_then(|name| name.to_str())?.to_owned();

        Some(Repo::new(name, path))
    }
}
