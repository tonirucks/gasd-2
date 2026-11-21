use super::{AppState, Result, with_service};
use gasd_core::{Task, TaskList, TaskUpdate};
use tauri::State;

#[tauri::command]
pub async fn task_folders_get(state: State<'_, AppState>) -> Result<Vec<String>> {
    with_service(state, |service| service.get_task_folders()).await
}

#[tauri::command]
pub async fn tasks_link_folder(
    state: State<'_, AppState>,
    id: i64,
    folder: Option<String>,
) -> Result<Task> {
    with_service(state, move |service| {
        service.link_task_folder(id, folder.as_deref())
    })
    .await
}

#[tauri::command]
pub async fn subtasks_create(
    state: State<'_, AppState>,
    task_id: i64,
    title: String,
) -> Result<Task> {
    with_service(state, move |service| {
        service.create_subtask(task_id, &title)
    })
    .await
}

#[tauri::command]
pub async fn subtasks_update(
    state: State<'_, AppState>,
    task_id: i64,
    id: i64,
    title: String,
    completed: bool,
) -> Result<Task> {
    with_service(state, move |service| {
        service.update_subtask(task_id, id, &title, completed)
    })
    .await
}

#[tauri::command]
pub async fn subtasks_delete(state: State<'_, AppState>, task_id: i64, id: i64) -> Result<Task> {
    with_service(state, move |service| service.delete_subtask(task_id, id)).await
}

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
