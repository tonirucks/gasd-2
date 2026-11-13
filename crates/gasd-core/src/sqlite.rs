use std::{collections::HashSet, path::Path, time::Duration};

use rusqlite::{Connection, TransactionBehavior};

use crate::{Error, Result};

const SCHEMA_VERSION: i64 = 1;

impl From<rusqlite::Error> for Error {
    fn from(error: rusqlite::Error) -> Self {
        Self::Storage(Box::new(error))
    }
}

pub struct SqliteRepository {
    pub(crate) connection: Connection,
}

impl SqliteRepository {
    /// Opens an explicit path. The caller owns path selection and directory creation.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::initialize(Connection::open(path)?)
    }

    pub fn in_memory() -> Result<Self> {
        Self::initialize(Connection::open_in_memory()?)
    }

    fn initialize(mut connection: Connection) -> Result<Self> {
        connection.busy_timeout(Duration::from_secs(5))?;
        connection.pragma_update(None, "foreign_keys", true)?;
        // Lock before reading the version so simultaneous desktop/CLI startups
        // cannot both attempt the same migration.
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let version = transaction.pragma_query_value(None, "user_version", |row| row.get(0))?;
        if version > SCHEMA_VERSION {
            return Err(Error::UnsupportedSchema {
                found: version,
                supported: SCHEMA_VERSION,
            });
        }
        if version < 1 {
            transaction.execute_batch(include_str!("../migrations/001_initial.sql"))?;
            transaction.pragma_update(None, "user_version", SCHEMA_VERSION)?;
        }
        transaction.commit()?;
        Ok(Self { connection })
    }
}

pub(crate) fn require_changed(changed: usize, entity: &'static str, id: i64) -> Result<()> {
    if changed == 0 {
        Err(Error::NotFound { entity, id })
    } else {
        Ok(())
    }
}

pub(crate) fn validate_order(existing: &[i64], ids: &[i64]) -> Result<()> {
    let supplied: HashSet<_> = ids.iter().copied().collect();
    if ids.len() != existing.len()
        || supplied.len() != ids.len()
        || existing.iter().any(|id| !supplied.contains(id))
    {
        Err(Error::InvalidOrder)
    } else {
        Ok(())
    }
}
