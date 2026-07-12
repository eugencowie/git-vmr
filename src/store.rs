//! FileStore: the single owner of one persistent TOML file. It holds the
//! file's contents in memory, knows the file's path, and is the only thing
//! that reads or writes it.

use anyhow::{Context, Result};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fs;
use std::io::{ErrorKind, Write};
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct FileStore<T>
{
    /// Path to the stored file
    path: PathBuf,

    /// In-memory contents of the stored file
    data: T,

    /// What the file held at load time; [`FileStore::save`] writes only when
    /// the data differs. `None` poisons the snapshot so the next save always
    /// writes — used to replace an unreadable file with defaults.
    snapshot: Option<T>
}

impl<T> FileStore<T>
where T: Serialize + DeserializeOwned + Default + Clone + PartialEq
{
    /// Load strictly: a malformed or unreadable file is an error, a missing
    /// one means defaults.
    pub fn load(path: PathBuf) -> Result<Self>
    {
        match fs::read_to_string(&path)
        {
            Ok(contents) =>
            {
                let data = toml::from_str(&contents).with_context(|| {
                    format!("failed to parse {}", path.display())
                })?;
                Ok(Self::clean(path, data))
            }

            // Missing file means defaults
            Err(err) if err.kind() == ErrorKind::NotFound =>
                Ok(Self::clean(path, T::default())),

            Err(err) => Err(err)
                .with_context(|| format!("failed to read {}", path.display()))
        }
    }

    /// Load forgivingly: a malformed or unreadable file means defaults plus
    /// a warning, and the next save replaces the file. A missing one means
    /// defaults with no warning.
    pub fn load_or_default(path: PathBuf) -> (Self, Option<String>)
    {
        match fs::read_to_string(&path)
        {
            Ok(contents) => match toml::from_str(&contents)
            {
                Ok(data) => (Self::clean(path, data), None),
                Err(err) =>
                {
                    let warning = format!(
                        "failed to parse {}: {err:#}; using defaults",
                        path.display()
                    );
                    (Self::poisoned(path), Some(warning))
                }
            },

            // Missing file means defaults
            Err(err) if err.kind() == ErrorKind::NotFound =>
                (Self::clean(path, T::default()), None),

            Err(err) =>
            {
                let warning = format!(
                    "failed to read {}: {err:#}; using defaults",
                    path.display()
                );
                (Self::poisoned(path), Some(warning))
            }
        }
    }

    /// Store for data already in memory; save writes only after changes.
    pub fn new(path: PathBuf, data: T) -> Self
    {
        Self { path, snapshot: Some(data.clone()), data }
    }

    /// Save the data, writing only when it differs from what was loaded.
    pub fn save(&mut self) -> Result<()>
    {
        // Skip write when the file already holds this data
        if self.snapshot.as_ref() == Some(&self.data)
        {
            return Ok(());
        }

        self.write()
    }

    /// Write the data unconditionally, creating the file if needed.
    pub fn write(&mut self) -> Result<()>
    {
        let parent = self
            .path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));

        // Create parent directory if needed
        fs::create_dir_all(parent).with_context(|| {
            format!("failed to create {}", parent.display())
        })?;

        // Serialize, then atomically replace the file from the same directory
        let contents = toml::to_string(&self.data).with_context(|| {
            format!("failed to serialize {}", self.path.display())
        })?;
        let mut temporary = tempfile::NamedTempFile::new_in(parent)
            .with_context(|| {
                format!(
                    "failed to create temporary file in {}",
                    parent.display()
                )
            })?;
        temporary.write_all(contents.as_bytes()).with_context(|| {
            format!("failed to write {}", self.path.display())
        })?;
        temporary.flush().with_context(|| {
            format!("failed to flush {}", self.path.display())
        })?;
        temporary.persist(&self.path).map_err(|err| err.error).with_context(
            || format!("failed to replace {}", self.path.display())
        )?;
        self.snapshot = Some(self.data.clone());
        Ok(())
    }

    /// Store whose snapshot matches the file on disk
    fn clean(path: PathBuf, data: T) -> Self
    {
        Self { path, snapshot: Some(data.clone()), data }
    }

    /// Defaulted store whose next save always writes
    fn poisoned(path: PathBuf) -> Self
    {
        Self { path, data: T::default(), snapshot: None }
    }
}

impl<T> Deref for FileStore<T>
{
    type Target = T;

    fn deref(&self) -> &T
    {
        &self.data
    }
}

impl<T> DerefMut for FileStore<T>
{
    fn deref_mut(&mut self) -> &mut T
    {
        &mut self.data
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(default)]
    struct Fixture
    {
        name: String,
        count: u32
    }

    fn path(tmp: &tempfile::TempDir) -> PathBuf
    {
        tmp.path().join("nested").join("fixture.toml")
    }

    #[test]
    fn roundtrips()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let mut store = FileStore::<Fixture>::load(path(&tmp)).unwrap();
        store.name = "vmr".to_owned();
        store.count = 3;

        // Act
        store.save().unwrap();
        let loaded = FileStore::<Fixture>::load(path(&tmp)).unwrap();

        // Assert
        assert_eq!(*loaded, *store);
    }

    #[test]
    fn missing_file_loads_defaults()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();

        // Act
        let strict = FileStore::<Fixture>::load(path(&tmp)).unwrap();
        let (forgiving, warning) =
            FileStore::<Fixture>::load_or_default(path(&tmp));

        // Assert
        assert_eq!(*strict, Fixture::default());
        assert_eq!(*forgiving, Fixture::default());
        assert_eq!(warning, None);
    }

    #[test]
    fn strict_load_rejects_malformed_file()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("fixture.toml");
        fs::write(&path, "count =").unwrap();

        // Act
        let err = FileStore::<Fixture>::load(path).unwrap_err();

        // Assert
        assert!(err.to_string().contains("failed to parse"));
    }

    #[test]
    fn forgiving_load_recovers_from_malformed_file()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("fixture.toml");
        fs::write(&path, "count =").unwrap();

        // Act
        let (mut store, warning) =
            FileStore::<Fixture>::load_or_default(path.clone());

        // Assert
        assert_eq!(*store, Fixture::default());
        assert!(warning.unwrap().contains("failed to parse"));

        // The poisoned snapshot replaces the invalid file on save
        store.save().unwrap();
        let loaded = FileStore::<Fixture>::load(path).unwrap();
        assert_eq!(*loaded, Fixture::default());
    }

    #[test]
    fn save_skips_write_when_unchanged()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let mut store = FileStore::<Fixture>::load(path(&tmp)).unwrap();

        // Act: untouched defaults never create a file
        store.save().unwrap();

        // Assert
        assert!(!path(&tmp).exists());
    }

    #[test]
    fn write_creates_file_unconditionally()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let mut store = FileStore::<Fixture>::load(path(&tmp)).unwrap();

        // Act
        store.write().unwrap();

        // Assert
        assert_eq!(
            fs::read_to_string(path(&tmp)).unwrap(),
            toml::to_string(&Fixture::default()).unwrap()
        );
    }
}
