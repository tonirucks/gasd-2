use super::{AppState, Result, with_service};
use gasd_core::Section;
use tauri::State;

#[tauri::command]
pub async fn sections_create(state: State<'_, AppState>, name: String) -> Result<Section> {
    with_service(state, move |service| service.create_section(&name)).await
}

#[tauri::command]
pub async fn sections_rename(state: State<'_, AppState>, id: i64, name: String) -> Result<Section> {
    with_service(state, move |service| service.rename_section(id, &name)).await
}

#[tauri::command]
pub async fn sections_delete(state: State<'_, AppState>, id: i64) -> Result<()> {
    with_service(state, move |service| service.delete_section(id)).await
}

#[tauri::command]
pub async fn sections_reorder(state: State<'_, AppState>, ids: Vec<i64>) -> Result<()> {
    with_service(state, move |service| service.reorder_sections(&ids)).await
}
