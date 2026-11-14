use super::{AppState, Result, with_service};
use gasd_core::{Task, TaskList, TaskUpdate};
use tauri::State;

#[tauri::command]
pub async fn tasks_create(
    state: State<'_, AppState>,
    section_id: i64,
    title: String,
) -> Result<Task> {
    with_service(state, move |service| {
        service.create_task(section_id, &title)
    })
    .await
}

#[tauri::command]
pub async fn tasks_update(state: State<'_, AppState>, id: i64, update: TaskUpdate) -> Result<Task> {
    with_service(state, move |service| service.update_task(id, update)).await
}

#[tauri::command]
pub async fn tasks_delete(state: State<'_, AppState>, id: i64) -> Result<()> {
    with_service(state, move |service| service.delete_task(id)).await
}

#[tauri::command]
pub async fn tasks_reorder(
    state: State<'_, AppState>,
    list: TaskList,
    ids: Vec<i64>,
) -> Result<()> {
    with_service(state, move |service| service.reorder_tasks(list, &ids)).await
}

#[tauri::command]
pub async fn tasks_place(
    state: State<'_, AppState>,
    id: i64,
    list: TaskList,
    ids: Vec<i64>,
) -> Result<()> {
    with_service(state, move |service| service.place_task(id, list, &ids)).await
}
