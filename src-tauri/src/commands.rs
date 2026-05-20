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

#[tauri::command]
pub async fn get_app_zoom(state: tauri::State<'_, AppState>) -> Result<f64, String> {
    let val = db::settings::get(&state.db, "app_zoom").await?;
    Ok(val.and_then(|s| s.parse::<f64>().ok()).unwrap_or(1.3))
}

#[tauri::command]
pub async fn set_app_zoom(
    state: tauri::State<'_, AppState>,
    window: tauri::WebviewWindow,
    zoom: f64,
) -> Result<(), String> {
    let zoom = zoom.clamp(0.5, 3.0);
    db::settings::set(&state.db, "app_zoom", &zoom.to_string()).await?;
    window.set_zoom(zoom).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_room_groups(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let val = db::settings::get(&state.db, "room_groups").await?;
    Ok(val.unwrap_or_else(|| "{}".to_string()))
}

#[tauri::command]
pub async fn set_room_groups(state: tauri::State<'_, AppState>, groups: String) -> Result<(), String> {
    db::settings::set(&state.db, "room_groups", &groups).await
}

#[tauri::command]
pub async fn get_diagnosis_vocab(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let val = db::settings::get(&state.db, "diagnosis_vocab").await?;
    Ok(val.unwrap_or_else(|| "[]".to_string()))
}

#[tauri::command]
pub async fn set_diagnosis_vocab(state: tauri::State<'_, AppState>, vocab: String) -> Result<(), String> {
    db::settings::set(&state.db, "diagnosis_vocab", &vocab).await
}

#[tauri::command]
pub async fn get_procedure_vocab(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let val = db::settings::get(&state.db, "procedure_vocab").await?;
    Ok(val.unwrap_or_else(|| "[]".to_string()))
}

#[tauri::command]
pub async fn set_procedure_vocab(state: tauri::State<'_, AppState>, vocab: String) -> Result<(), String> {
    db::settings::set(&state.db, "procedure_vocab", &vocab).await
}

#[tauri::command]
pub async fn get_room_code_map(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let val = db::settings::get(&state.db, "room_code_map").await?;
    Ok(val.unwrap_or_else(|| "{}".to_string()))
}

#[tauri::command]
pub async fn set_room_code_map(state: tauri::State<'_, AppState>, map: String) -> Result<(), String> {
    db::settings::set(&state.db, "room_code_map", &map).await
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

fn is_local_anes_only(shift: &RoomScheduleEntry) -> bool {
    let text = format!("{} {}", shift.dept, shift.notes);
    text.contains("局麻") || text.contains("局部")
}

fn requires_general_anes(anesthesia: &str) -> bool {
    matches!(anesthesia, "全身麻醉" | "半身麻醉")
}

// Fuzzy dept match: exact OR one string contains the other (min 6 bytes ≈ 2 CJK chars to avoid single-char false positives)
fn dept_matches(a: &str, b: &str) -> bool {
    let a = a.trim();
    let b = b.trim();
    if a.is_empty() || b.is_empty() { return false; }
    a == b || (b.len() >= 6 && a.contains(b)) || (a.len() >= 6 && b.contains(a))
}

#[tauri::command]
pub async fn get_room_recommendation(
    state: tauri::State<'_, AppState>,
    urgency: String,
    dept: String,
    anesthesia: String,
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
        .find(|r| dept_matches(&r.dept, &dept))
        .map(|r| r.preferred_rooms.clone())
        .unwrap_or_default();

    let req_general = requires_general_anes(&anesthesia);
    let is_urgent   = matches!(urgency.as_str(), "Trauma" | "Level1" | "Level2" | "Level3");

    let mut results: Vec<RoomRecommendation> = rooms.iter()
        .filter(|room| room.is_active)   // 停用房間不列入推薦
        .map(|room| {
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

        // 今日該房間所有時段
        let room_shifts: Vec<&RoomScheduleEntry> = shifts.iter()
            .filter(|s| s.room_name == room.name).collect();
        let has_any_shift = !room_shifts.is_empty();

        // 急診房：今日有 is_emergency 時段
        let is_emergency_room = room_shifts.iter().any(|s| s.is_emergency);

        // 麻醉衝突：全麻/半麻刀 vs 局麻房
        let has_local_only_shift = room_shifts.iter().any(|s| is_local_anes_only(s));
        let anes_conflict = req_general && has_local_only_shift;

        // 科別符合（偏好房間 or 當日科別時段吻合），衝突時取消
        let raw_dept_match = preferred_rooms.contains(&room.name) ||
            shifts.iter().any(|s| s.room_name == room.name && dept_matches(&s.dept, &dept));
        let dept_match = raw_dept_match && !anes_conflict;

        let room_assignments: Vec<&StaffAssignment> = assignments.iter()
            .filter(|a| a.room_name == room.name).collect();
        let has_scrub = room_assignments.iter().any(|a| a.role == "Scrub");
        let has_circ  = room_assignments.iter().any(|a| a.role == "Circ");
        let has_sa    = room_assignments.iter().any(|a| a.role == "SA");
        let has_staff = has_scrub && has_circ;

        let mut score: i64 = 0;
        if dept_match      { score += 5000; }
        if is_available    { score += 4000; }
        if within_deadline { score += 3000; }
        if has_staff       { score += 1000; }
        if has_sa          { score += 200; }
        if has_any_shift && !dept_match && !is_emergency_room { score -= 500; }
        if room.is_backup  { score -= 1000; }
        // 麻醉衝突：強力降分，確保排在局麻房之後
        if anes_conflict   { score -= 6000; }
        // 急診房加分：緊急刀（非Normal）且需全麻/半麻時才加分，避免常規待排被排入急診房
        if is_emergency_room && req_general && is_urgent { score += 2000; }

        let avail_str = if is_available {
            "空刀房".to_string()
        } else if est_available_mins > 0 {
            let h = est_available_mins / 60;
            let m = est_available_mins % 60;
            if h > 0 { format!("約 {}h{}m 後空出", h, m) }
            else { format!("約 {}m 後空出", m) }
        } else {
            "忙碌中".to_string()
        };
        let dept_str = if anes_conflict {
            " · ⚠ 局麻房，麻醉不符".to_string()
        } else if dept_match {
            " · 科別符合".to_string()
        } else if is_emergency_room {
            " · 急診房".to_string()
        } else if has_any_shift {
            let depts: Vec<&str> = room_shifts.iter().map(|s| s.dept.as_str()).collect();
            format!(" · 排班：{}", depts.join("/"))
        } else {
            String::new()
        };
        let reason = format!("{}{}", avail_str, dept_str);

        RoomRecommendation {
            room_name: room.name.clone(),
            score,
            is_available,
            dept_match,
            has_staff,
            within_deadline,
            est_available_mins,
            reason,
            anes_conflict,
            is_emergency_room,
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
