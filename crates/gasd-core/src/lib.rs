//! Shared application logic and persistence, independent of any UI or CLI.
//!
//! ```
//! use gasd_core::{Service, SqliteRepository};
//!
//! # fn main() -> gasd_core::Result<()> {
//! let mut app = Service::new(SqliteRepository::in_memory()?);
//! let section = app.create_section("Personal")?;
//! let task = app.create_task(section.id, "Read a chapter")?;
//! assert_eq!(app.get_task(task.id)?.title, "Read a chapter");
//! # Ok(())
//! # }
//! ```

pub mod error;
pub mod jira;
pub mod model;
pub mod sections;
pub mod service;
pub mod settings;
pub mod sqlite;
pub mod storage;
pub mod tasks;
mod validation;

pub use error::{Error, Result};
pub use model::{Section, Task, TaskList, TaskStatus, TaskUpdate};
pub use service::Service;
pub use sqlite::SqliteRepository;
