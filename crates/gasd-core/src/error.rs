pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("folder must be an existing directory with a valid Unicode path")]
    InvalidFolder,
    #[error("{field} must not be blank")]
    Validation { field: &'static str },
    #[error("{entity} {id} was not found")]
    NotFound { entity: &'static str, id: i64 },
    #[error("order must contain every item in the list exactly once")]
    InvalidOrder,
    #[error("database schema version {found} is newer than supported version {supported}")]
    UnsupportedSchema { found: i64, supported: i64 },
    #[error("database error: {0}")]
    Storage(#[source] Box<dyn std::error::Error + Send + Sync>),
}
