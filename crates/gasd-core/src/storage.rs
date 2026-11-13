//! Shared native storage location for the CLI and desktop adapters.

use std::path::{Path, PathBuf};

use crate::{Service, SqliteRepository};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not determine the user's local data directory; set GASD_DATABASE")]
    NoDataDirectory,
    #[error("database path must not be empty")]
    EmptyPath,
    #[error("could not create database directory {path}: {source}")]
    CreateDirectory {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error(transparent)]
    Core(#[from] crate::Error),
}

/// Uses GASD_DATABASE when set, otherwise the platform's local data directory.
/// Neither the working directory nor the desktop bundle identifier affects it.
pub fn database_path() -> Result<PathBuf, Error> {
    if let Some(path) = std::env::var_os("GASD_DATABASE") {
        if path.is_empty() {
            return Err(Error::EmptyPath);
        }
        return Ok(path.into());
    }
    default_database_path(dirs::data_local_dir())
}

fn default_database_path(directory: Option<PathBuf>) -> Result<PathBuf, Error> {
    Ok(directory
        .ok_or(Error::NoDataDirectory)?
        .join("gasd")
        .join("gasd.sqlite3"))
}

/// Creates parent directories and opens/migrates a database using core's locking.
pub fn open(path: impl AsRef<Path>) -> Result<Service, Error> {
    let path = path.as_ref();
    if path.as_os_str().is_empty() {
        return Err(Error::EmptyPath);
    }
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|source| Error::CreateDirectory {
            path: parent.to_owned(),
            source,
        })?;
    }
    Ok(Service::new(SqliteRepository::open(path)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_default_location() {
        assert_eq!(
            default_database_path(Some(PathBuf::from("data"))).unwrap(),
            PathBuf::from("data/gasd/gasd.sqlite3")
        );
        assert!(matches!(
            default_database_path(None),
            Err(Error::NoDataDirectory)
        ));
    }

    #[test]
    fn adapters_share_persistent_data() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested/gasd.sqlite3");
        let mut cli = open(&path).unwrap();
        let gui = open(&path).unwrap();
        let section = cli.create_section("Shared").unwrap();
        assert_eq!(gui.get_sections().unwrap(), vec![section]);
    }

    #[test]
    fn rejects_empty_path() {
        assert!(matches!(open(""), Err(Error::EmptyPath)));
    }
}
