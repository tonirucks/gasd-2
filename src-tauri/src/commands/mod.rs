mod jira;
mod sections;
mod settings;
mod tasks;

pub use jira::*;
pub use sections::*;
pub use settings::*;
pub use tasks::*;

use gasd_core::Service;
use serde::Serialize;
use std::sync::{Arc, Mutex};

pub type AppState = Arc<Mutex<Service>>;
pub type Result<T> = std::result::Result<T, CommandError>;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    code: &'static str,
    message: String,
}

impl From<gasd_core::Error> for CommandError {
    fn from(error: gasd_core::Error) -> Self {
        let code = match &error {
            gasd_core::Error::Validation { .. } => "validation",
            gasd_core::Error::NotFound { .. } => "notFound",
            gasd_core::Error::InvalidOrder => "invalidOrder",
            _ => "storage",
        };
        Self {
            code,
            message: error.to_string(),
        }
    }
}

// Keep SQLite's blocking IO and busy timeout off the UI and async runtime threads.
async fn with_service<T: Send + 'static>(
    state: tauri::State<'_, AppState>,
    operation: impl FnOnce(&mut Service) -> gasd_core::Result<T> + Send + 'static,
) -> Result<T> {
    with_native_service(state, move |service| operation(service).map_err(Into::into)).await
}

// Native settings operations also need credential-store errors, not only core errors.
async fn with_native_service<T: Send + 'static>(
    state: tauri::State<'_, AppState>,
    operation: impl FnOnce(&mut Service) -> Result<T> + Send + 'static,
) -> Result<T> {
    let state = Arc::clone(state.inner());
    tauri::async_runtime::spawn_blocking(move || {
        let mut service = state.lock().map_err(|_| CommandError {
            code: "internal",
            message: "The database is unavailable. Please restart the app.".into(),
        })?;
        operation(&mut service)
    })
    .await
    .map_err(|error| CommandError {
        code: "internal",
        message: format!("Database operation failed: {error}"),
    })?
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListData {
    sections: Vec<gasd_core::Section>,
    tasks: Vec<gasd_core::Task>,
}

#[tauri::command]
pub async fn list_get(state: tauri::State<'_, AppState>) -> Result<ListData> {
    with_service(state, |service| {
        Ok(ListData {
            sections: service.get_sections()?,
            tasks: service.get_tasks()?,
        })
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_errors_have_stable_ipc_codes() {
        for (error, code) in [
            (
                gasd_core::Error::Validation { field: "title" },
                "validation",
            ),
            (
                gasd_core::Error::NotFound {
                    entity: "task",
                    id: 42,
                },
                "notFound",
            ),
            (gasd_core::Error::InvalidOrder, "invalidOrder"),
        ] {
            let message = error.to_string();
            assert_eq!(
                serde_json::to_value(CommandError::from(error)).unwrap(),
                serde_json::json!({ "code": code, "message": message })
            );
        }
    }

    #[test]
    fn list_response_preserves_core_models() {
        let mut service = Service::new(gasd_core::SqliteRepository::in_memory().unwrap());
        let section = service.create_section("Work").unwrap();
        let task = service.create_task(section.id, "Review").unwrap();
        let response = ListData {
            sections: service.get_sections().unwrap(),
            tasks: service.get_tasks().unwrap(),
        };
        let json = serde_json::to_value(response).unwrap();
        assert_eq!(json["sections"][0], serde_json::to_value(section).unwrap());
        assert_eq!(json["tasks"][0], serde_json::to_value(task).unwrap());
        assert!(json["tasks"][0].get("sectionId").is_some());
    }
}
