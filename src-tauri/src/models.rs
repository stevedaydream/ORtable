use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UrgencyLevel {
    Trauma,
    Level1,
    Level2,
    Level3,
    Normal,
}

impl UrgencyLevel {
    pub fn base_weight(&self) -> i64 {
        match self {
            UrgencyLevel::Trauma => 100_000,
            UrgencyLevel::Level1 => 80_000,
            UrgencyLevel::Level2 => 60_000,
            UrgencyLevel::Level3 => 40_000,
            UrgencyLevel::Normal => 10_000,
        }
    }
}

impl Default for UrgencyLevel {
    fn default() -> Self {
        UrgencyLevel::Normal
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SurgeryTask {
    pub id: i64,
    pub seq_no: i32,
    pub patient_name: String,
    pub gender: String,
    pub age: i32,
    pub chart_no: String,
    pub bed_no: String,
    pub dept: String,
    pub diagnosis: String,
    pub body_part: String,
    pub procedure: String,
    pub anesthesia: String,
    pub surgeon: String,
    pub vs_note: String,
    pub expected_room: String,
    pub urgency: UrgencyLevel,
    pub scheduled_at: Option<i64>,
    pub created_at: i64,
    pub est_time_mins: i32,
    pub status: String,
    #[serde(default)]
    pub linked_task_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Staff {
    pub id: i64,
    pub name: String,
    pub role: String,
    #[serde(rename = "type")]
    pub staff_type: String,
    pub staff_category: String, // "sa" | "or_nurse" | "intern" | "cross_train"
    pub unit: String,           // unit name for cross_train staff
    pub is_on_call: bool,
    pub is_volunteer_extra: bool,
    pub today_shift_start: i64,
    pub next_day_shift_start: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub id: i64,
    pub name: String,
    pub display_order: i32,
    pub is_backup: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RoomScheduleEntry {
    pub id: i64,
    pub room_name: String,
    pub dept: String,
    pub date: String,      // "YYYY-MM-DD"
    pub start_time: i64,   // unix timestamp
    pub end_time: i64,     // unix timestamp
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StaffAssignment {
    pub id: i64,
    pub staff_name: String,
    pub room_name: String,
    pub date: String,   // "YYYY-MM-DD"
    pub role: String,   // "SA" | "Scrub" | "Circ" | "R" | "VS"
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StaffRosterEntry {
    pub id: i64,
    pub staff_name: String,
    pub date: String,       // "YYYY-MM-DD"
    pub shift_name: String, // e.g., "D", "N1", "OFF"
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShiftDefinition {
    pub name: String,        // e.g., "D"
    pub start_time: String,  // "07:30"
    pub end_time: String,    // "16:00"
    pub is_on_call: bool,    // 是否為值班/二線
}

/// Per-department priority adjustment and preferred room list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeptRule {
    pub dept: String,
    /// Flat score bonus added to all tasks of this dept.
    pub priority_bonus: i64,
    /// Rooms preferred by this dept (informational, used for room_bonus calculation).
    pub preferred_rooms: Vec<String>,
    /// Tailwind-style color class (e.g., "bg-blue-800 text-blue-200")
    pub color: String,
}

/// A task paired with its computed priority score and deadline status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskWithScore {
    pub task: SurgeryTask,
    pub score: i64,
    /// True if the task's deadline has already passed.
    pub is_overdue: bool,
    /// 0–100 (capped). None for Normal urgency (no hard deadline).
    pub deadline_elapsed_pct: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SelfPayItem {
    pub id: i64,
    pub name: String,
    pub price: i64,
    pub notes: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExtraComplianceResult {
    pub allowed: bool,
    pub reason: Option<String>,
}

/// Result of a batch Extra-line compliance check for one staff member.
#[derive(Debug, Serialize, Deserialize)]
pub struct StaffComplianceResult {
    pub staff_id: i64,
    pub allowed: bool,
    pub reason: Option<String>,
}

/// Room recommendation entry for the quick-assign wizard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomRecommendation {
    pub room_name: String,
    pub score: i64,
    pub is_available: bool,
    pub dept_match: bool,
    pub has_staff: bool,
    pub within_deadline: bool,
    pub est_available_mins: i64,
    pub reason: String,
}
