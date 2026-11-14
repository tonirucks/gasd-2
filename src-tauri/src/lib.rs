mod commands;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let service = gasd_core::storage::open(gasd_core::storage::database_path()?)?;
            app.manage(std::sync::Arc::new(std::sync::Mutex::new(service)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_get,
            commands::sections_create,
            commands::sections_rename,
            commands::sections_delete,
            commands::sections_reorder,
            commands::tasks_create,
            commands::tasks_update,
            commands::tasks_delete,
            commands::tasks_reorder,
            commands::tasks_place,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
