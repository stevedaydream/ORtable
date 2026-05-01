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
    pub patient_name: String,
    pub chart_no: String,
    pub procedure: String,
    pub diagnosis: String,
    pub vs_note: String,
    pub dept: String,
    pub expected_room: String,
    pub urgency: UrgencyLevel,
    pub scheduled_at: Option<i64>,
    pub created_at: i64,
    pub est_time_mins: i32,
    pub status: String,
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

/// Per-department priority adjustment and preferred room list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeptRule {
    pub dept: String,
    /// Flat score bonus added to all tasks of this dept.
    pub priority_bonus: i64,
    /// Rooms preferred by this dept (informational, used for room_bonus calculation).
    pub preferred_rooms: Vec<String>,
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
