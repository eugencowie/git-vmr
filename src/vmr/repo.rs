use std::path::PathBuf;

/// A child repo: its manifest path, as the label output is keyed by, and
/// where it lives on disk.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Repo
{
    pub name: String,
    pub path: PathBuf
}
