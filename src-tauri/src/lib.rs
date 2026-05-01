mod commands;
mod db;
mod engine;
mod models;
mod state;
mod sync;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::block_on(async move {
                let pool = db::init(&handle).await.expect("DB 初始化失敗");
                handle.manage(AppState { db: pool });
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Rule engine (stateless)
            commands::get_tasks_with_scores,
            commands::get_sorted_tasks,
            commands::check_extra_compliance,
            commands::batch_check_extra_compliance,
            // Tasks CRUD
            commands::get_all_tasks,
            commands::create_task,
            commands::update_task,
            commands::delete_task,
            // Staff CRUD
            commands::get_all_staff,
            commands::create_staff,
            commands::update_staff,
            commands::delete_staff,
            // Dept Rules
            commands::get_dept_rules,
            commands::upsert_dept_rule,
            commands::delete_dept_rule,
            // Rooms
            commands::get_all_rooms,
            commands::create_room,
            commands::update_room,
            commands::delete_room,
            // Room Shifts
            commands::get_room_shifts_by_date,
            commands::replace_room_shifts_by_month,
            // Settings
            commands::get_gas_url,
            commands::set_gas_url,
            // Sync
            commands::sync_push,
            commands::sync_pull,
            commands::get_sync_timestamps,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
