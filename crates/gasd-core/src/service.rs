use crate::SqliteRepository;

/// Application entry point shared by desktop and command-line adapters.
pub struct Service {
    pub(crate) repository: SqliteRepository,
}

impl Service {
    pub fn new(repository: SqliteRepository) -> Self {
        Self { repository }
    }
}
