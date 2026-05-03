use crate::{
    db, engine, sync,
    models::{DeptRule, ExtraComplianceResult, Room, RoomRecommendation, RoomScheduleEntry, SelfPayItem, Staff, StaffAssignment, StaffComplianceResult, StaffRosterEntry, SurgeryTask, TaskWithScore},
    state::AppState,
};

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

// ── Rule Engine (stateless) ───────────────────────────────────────────────────

#[tauri::command]
pub fn get_tasks_with_scores(
    tasks: Vec<SurgeryTask>,
    dept_rules: Vec<DeptRule>,
    available_rooms: Vec<String>,
) -> Vec<TaskWithScore> {
    engine::sort_tasks(tasks, now_secs(), &dept_rules, &available_rooms)
}

#[tauri::command]
pub fn get_sorted_tasks(tasks: Vec<SurgeryTask>) -> Vec<SurgeryTask> {
    engine::sort_tasks(tasks, now_secs(), &[], &[])
        .into_iter()
        .map(|ts| ts.task)
        .collect()
}

#[tauri::command]
pub fn check_extra_compliance(
    today_shift_start: i64,
    next_day_shift_start: i64,
    estimated_extra_end: i64,
) -> ExtraComplianceResult {
    engine::check_compliance(today_shift_start, next_day_shift_start, estimated_extra_end)
}

#[tauri::command]
pub fn batch_check_extra_compliance(
    staff_list: Vec<Staff>,
    estimated_extra_end: i64,
) -> Vec<StaffComplianceResult> {
    engine::batch_compliance(&staff_list, estimated_extra_end)
}

// ── Surgery Tasks CRUD ────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_all_tasks(state: tauri::State<'_, AppState>) -> Result<Vec<SurgeryTask>, String> {
    db::tasks::get_all(&state.db).await
}

#[tauri::command]
pub async fn create_task(
    state: tauri::State<'_, AppState>,
    task: SurgeryTask,
) -> Result<SurgeryTask, String> {
    db::tasks::create(&state.db, &task).await
}

#[tauri::command]
pub async fn update_task(
    state: tauri::State<'_, AppState>,
    task: SurgeryTask,
) -> Result<SurgeryTask, String> {
    db::tasks::update(&state.db, &task).await
}

#[tauri::command]
pub async fn delete_task(
    state: tauri::State<'_, AppState>,
    id: i64,
) -> Result<(), String> {
    db::tasks::delete(&state.db, id).await
}

#[tauri::command]
pub async fn batch_create_tasks(
    state: tauri::State<'_, AppState>,
    tasks: Vec<SurgeryTask>,
) -> Result<(), String> {
    db::tasks::batch_create(&state.db, &tasks).await
}

// ── Staff CRUD ────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_all_staff(state: tauri::State<'_, AppState>) -> Result<Vec<Staff>, String> {
    db::staff::get_all(&state.db).await
}

#[tauri::command]
pub async fn create_staff(
    state: tauri::State<'_, AppState>,
    staff: Staff,
) -> Result<Staff, String> {
    db::staff::create(&state.db, &staff).await
}

#[tauri::command]
pub async fn update_staff(
    state: tauri::State<'_, AppState>,
    staff: Staff,
) -> Result<Staff, String> {
    db::staff::update(&state.db, &staff).await
}

#[tauri::command]
pub async fn delete_staff(
    state: tauri::State<'_, AppState>,
    id: i64,
) -> Result<(), String> {
    db::staff::delete(&state.db, id).await
}

// ── Dept Rules CRUD ───────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_dept_rules(state: tauri::State<'_, AppState>) -> Result<Vec<DeptRule>, String> {
    db::dept_rules::get_all(&state.db).await
}

#[tauri::command]
pub async fn upsert_dept_rule(
    state: tauri::State<'_, AppState>,
    rule: DeptRule,
) -> Result<DeptRule, String> {
    db::dept_rules::upsert(&state.db, &rule).await
}

#[tauri::command]
pub async fn delete_dept_rule(
    state: tauri::State<'_, AppState>,
    dept: String,
) -> Result<(), String> {
    db::dept_rules::delete(&state.db, &dept).await
}

// ── Staff Roster & Shifts ──────────────────────────────────────────────────
#[tauri::command]
pub async fn get_staff_roster_by_date(state: tauri::State<'_, AppState>, date: String) -> Result<Vec<StaffRosterEntry>, String> {
    db::staff_roster::get_by_date(&state.db, &date).await
}

#[tauri::command]
pub async fn replace_staff_roster_by_month(state: tauri::State<'_, AppState>, month: String, entries: Vec<StaffRosterEntry>) -> Result<(), String> {
    db::staff_roster::replace_by_month(&state.db, &month, &entries).await
}

// ── Sync ───────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_gas_url(state: tauri::State<'_, AppState>) -> Result<Option<String>, String> {
    db::settings::get(&state.db, "gas_url").await
}

#[tauri::command]
pub async fn set_gas_url(
    state: tauri::State<'_, AppState>,
    url: String,
) -> Result<(), String> {
    db::settings::set(&state.db, "gas_url", &url).await
}

// ── Rooms CRUD ────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_all_rooms(state: tauri::State<'_, AppState>) -> Result<Vec<Room>, String> {
    db::rooms::get_all(&state.db).await
}

#[tauri::command]
pub async fn create_room(
    state: tauri::State<'_, AppState>,
    room: Room,
) -> Result<Room, String> {
    db::rooms::create(&state.db, &room).await
}

#[tauri::command]
pub async fn update_room(
    state: tauri::State<'_, AppState>,
    room: Room,
) -> Result<Room, String> {
    db::rooms::update(&state.db, &room).await
}

#[tauri::command]
pub async fn delete_room(
    state: tauri::State<'_, AppState>,
    id: i64,
) -> Result<(), String> {
    db::rooms::delete(&state.db, id).await
}

// ── Room Shifts ───────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_room_shifts_by_date(
    state: tauri::State<'_, AppState>,
    date: String,
) -> Result<Vec<RoomScheduleEntry>, String> {
    db::room_shifts::get_by_date(&state.db, &date).await
}

#[tauri::command]
pub async fn replace_room_shifts_by_month(
    state: tauri::State<'_, AppState>,
    month: String,
    entries: Vec<RoomScheduleEntry>,
) -> Result<(), String> {
    db::room_shifts::replace_by_month(&state.db, &month, &entries).await
}

#[tauri::command]
pub async fn replace_room_shifts_by_date(
    state: tauri::State<'_, AppState>,
    date: String,
    entries: Vec<RoomScheduleEntry>,
) -> Result<(), String> {
    db::room_shifts::replace_by_date(&state.db, &date, &entries).await
}

// ── Staff Assignments ─────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_staff_assignments_by_date(
    state: tauri::State<'_, AppState>,
    date: String,
) -> Result<Vec<StaffAssignment>, String> {
    db::staff_assignments::get_by_date(&state.db, &date).await
}

#[tauri::command]
pub async fn replace_staff_assignments_by_month(
    state: tauri::State<'_, AppState>,
    month: String,
    entries: Vec<StaffAssignment>,
) -> Result<(), String> {
    db::staff_assignments::replace_by_month(&state.db, &month, &entries).await
}

#[tauri::command]
pub async fn add_staff_assignment(
    state: tauri::State<'_, AppState>,
    entry: StaffAssignment,
) -> Result<StaffAssignment, String> {
    db::staff_assignments::add_one(&state.db, &entry).await
}

#[tauri::command]
pub async fn remove_staff_assignment(
    state: tauri::State<'_, AppState>,
    id: i64,
) -> Result<(), String> {
    db::staff_assignments::remove_one(&state.db, id).await
}

// ── Self-Pay Items CRUD ───────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_all_selfpay_items(state: tauri::State<'_, AppState>) -> Result<Vec<SelfPayItem>, String> {
    db::selfpay_items::get_all(&state.db).await
}

#[tauri::command]
pub async fn create_selfpay_item(
    state: tauri::State<'_, AppState>,
    item: SelfPayItem,
) -> Result<SelfPayItem, String> {
    db::selfpay_items::create(&state.db, &item).await
}

#[tauri::command]
pub async fn update_selfpay_item(
    state: tauri::State<'_, AppState>,
    item: SelfPayItem,
) -> Result<SelfPayItem, String> {
    db::selfpay_items::update(&state.db, &item).await
}

#[tauri::command]
pub async fn delete_selfpay_item(
    state: tauri::State<'_, AppState>,
    id: i64,
) -> Result<(), String> {
    db::selfpay_items::delete(&state.db, id).await
}

// ── Quick Room Recommendation ─────────────────────────────────────────────────

#[tauri::command]
pub async fn get_room_recommendation(
    state: tauri::State<'_, AppState>,
    urgency: String,
    dept: String,
    est_mins: i64,
    date: String,
) -> Result<Vec<RoomRecommendation>, String> {
    let rooms       = db::rooms::get_all(&state.db).await?;
    let tasks       = db::tasks::get_all(&state.db).await?;
    let dept_rules  = db::dept_rules::get_all(&state.db).await?;
    let shifts      = db::room_shifts::get_by_date(&state.db, &date).await?;
    let assignments = db::staff_assignments::get_by_date(&state.db, &date).await?;

    let now = now_secs();

    let deadline_secs: Option<i64> = match urgency.as_str() {
        "Trauma" => Some(30 * 60),
        "Level1" => Some(2 * 3600),
        "Level2" => Some(6 * 3600),
        "Level3" => Some(24 * 3600),
        _ => None,
    };

    let preferred_rooms: Vec<String> = dept_rules
        .iter()
        .find(|r| r.dept == dept)
        .map(|r| r.preferred_rooms.clone())
        .unwrap_or_default();

    let mut results: Vec<RoomRecommendation> = rooms.iter().map(|room| {
        let active_tasks: Vec<&SurgeryTask> = tasks.iter().filter(|t| {
            t.expected_room == room.name &&
            matches!(t.status.as_str(), "scheduled" | "called" | "in_surgery")
        }).collect();

        let currently_occupied = active_tasks.iter().any(|t| {
            matches!(t.status.as_str(), "called" | "in_surgery")
        });
        let is_available = !currently_occupied;

        let est_free_at = active_tasks.iter()
            .filter_map(|t| t.scheduled_at.map(|sa| sa + (t.est_time_mins as i64) * 60))
            .max()
            .unwrap_or(now);
        let wait_secs = (est_free_at - now).max(0);
        let est_available_mins = wait_secs / 60;

        let within_deadline = deadline_secs
            .map(|d| wait_secs + est_mins * 60 <= d)
            .unwrap_or(true);

        let dept_match = preferred_rooms.contains(&room.name) ||
            shifts.iter().any(|s| s.room_name == room.name && s.dept == dept);

        let room_assignments: Vec<&StaffAssignment> = assignments.iter()
            .filter(|a| a.room_name == room.name).collect();
        let has_scrub = room_assignments.iter().any(|a| a.role == "Scrub");
        let has_circ  = room_assignments.iter().any(|a| a.role == "Circ");
        let has_staff = has_scrub && has_circ;

        let mut score: i64 = 0;
        if dept_match      { score += 5000; }
        if is_available    { score += 4000; }
        if within_deadline { score += 3000; }
        if has_staff       { score += 1000; }
        if room.is_backup  { score -= 1000; }

        let reason = if is_available {
            if dept_match { "空刀房 · 科別符合".to_string() }
            else { "空刀房".to_string() }
        } else if est_available_mins > 0 {
            let h = est_available_mins / 60;
            let m = est_available_mins % 60;
            if h > 0 { format!("約 {}h{}m 後空出", h, m) }
            else { format!("約 {}m 後空出", m) }
        } else {
            "忙碌中".to_string()
        };

        RoomRecommendation {
            room_name: room.name.clone(),
            score,
            is_available,
            dept_match,
            has_staff,
            within_deadline,
            est_available_mins,
            reason,
        }
    }).collect();

    results.sort_by(|a, b| b.score.cmp(&a.score));
    Ok(results)
}

// ── Cloud Sync ────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn sync_push(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let url = db::settings::get(&state.db, "gas_url")
        .await?
        .ok_or("尚未設定 GAS URL，請至設定頁面輸入")?;
    sync::push(&state.db, &url).await
}

#[tauri::command]
pub async fn sync_pull(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let url = db::settings::get(&state.db, "gas_url")
        .await?
        .ok_or("尚未設定 GAS URL，請至設定頁面輸入")?;
    sync::pull(&state.db, &url).await
}

#[tauri::command]
pub async fn get_sync_timestamps(
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let push_at = db::settings::get(&state.db, "last_push_at").await?;
    let pull_at = db::settings::get(&state.db, "last_pull_at").await?;
    Ok(serde_json::json!({
        "last_push_at": push_at.and_then(|s| s.parse::<i64>().ok()),
        "last_pull_at": pull_at.and_then(|s| s.parse::<i64>().ok()),
    }))
}
