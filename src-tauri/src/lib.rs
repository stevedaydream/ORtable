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
            let win = app.get_webview_window("main");
            let handle = app.handle().clone();
            tauri::async_runtime::block_on(async move {
                let pool = db::init(&handle).await.expect("DB 初始化失敗");
                let zoom: f64 = db::settings::get(&pool, "app_zoom")
                    .await
                    .ok()
                    .flatten()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(1.3);
                if let Some(w) = win {
                    let _ = w.set_zoom(zoom);
                }
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
            commands::batch_create_tasks,
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
            commands::replace_room_shifts_by_date,
            // Staff Assignments
            commands::get_staff_assignments_by_date,
            commands::replace_staff_assignments_by_month,
            commands::add_staff_assignment,
            commands::remove_staff_assignment,
            // Settings
            commands::get_gas_url,
            commands::set_gas_url,
            commands::get_app_zoom,
            commands::set_app_zoom,
            commands::get_room_code_map,
            commands::set_room_code_map,
            commands::get_room_groups,
            commands::set_room_groups,
            commands::get_diagnosis_vocab,
            commands::set_diagnosis_vocab,
            commands::get_procedure_vocab,
            commands::set_procedure_vocab,
            // Staff Roster
            commands::get_staff_roster_by_date,
            commands::replace_staff_roster_by_month,
            // Self-Pay Items
            commands::get_all_selfpay_items,
            commands::create_selfpay_item,
            commands::update_selfpay_item,
            commands::delete_selfpay_item,
            // Sync
            commands::sync_push,
            commands::sync_pull,
            commands::get_sync_timestamps,
            // Room Recommendation
            commands::get_room_recommendation,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
