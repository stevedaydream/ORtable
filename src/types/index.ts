export type UrgencyLevel = "Trauma" | "Level1" | "Level2" | "Level3" | "Normal";
export type TaskStatus = "waiting" | "scheduled" | "called" | "in_surgery" | "recovery";

export const TASK_STATUS_LABELS: Record<TaskStatus, string> = {
  waiting:    "待排",
  scheduled:  "已排程",
  called:     "已叫刀",
  in_surgery: "手術中",
  recovery:   "恢復室",
};

export const TASK_STATUS_COLORS: Record<TaskStatus, string> = {
  waiting:    "bg-gray-700 text-gray-300",
  scheduled:  "bg-blue-800 text-blue-200",
  called:     "bg-amber-800 text-amber-200",
  in_surgery: "bg-green-800 text-green-200",
  recovery:   "bg-purple-800 text-purple-200",
};
export type StaffRole = "R" | "Scrub" | "Circ" | "VS" | "SA";
export type StaffType = "doc" | "nur";
export type StaffCategory = "vs" | "r" | "sa" | "or_nurse" | "intern" | "cross_train";

export const STAFF_CATEGORY_LABELS: Record<StaffCategory, string> = {
  vs:          "主治醫師",
  r:           "住院醫師",
  sa:          "專責護理師",
  or_nurse:    "開刀房護理師",
  intern:      "實習生",
  cross_train: "其他單位待訓練",
};

export interface SurgeryTask {
  id: number;
  seq_no: number;
  patient_name: string;
  gender: string;
  age: number;
  chart_no: string;
  bed_no: string;
  dept: string;
  diagnosis: string;
  body_part: string;
  procedure: string;
  anesthesia: string;
  surgeon: string;
  vs_note: string;
  expected_room: string;
  urgency: UrgencyLevel;
  scheduled_at: number | null;
  created_at: number;
  est_time_mins: number;
  status: string;
}

export interface Staff {
  id: number;
  name: string;
  role: StaffRole;
  type: StaffType;
  staff_category: StaffCategory;
  unit: string;
  is_on_call: boolean;
  is_volunteer_extra: boolean;
  today_shift_start: number;
  next_day_shift_start: number;
}

export interface Room {
  id: number;
  name: string;
  display_order: number;
  is_backup: boolean;
}

export interface StaffAssignment {
  id: number;
  staff_name: string;
  room_name: string;
  date: string;   // "YYYY-MM-DD"
  role: string;   // "SA" | "Scrub" | "Circ" | "R" | "VS"
}

export interface StaffRosterEntry {
  id: number;
  staff_name: string;
  date: string;
  shift_name: string;
}

export interface ShiftDefinition {
  name: string;
  start_time: string;
  end_time: string;
  is_on_call: boolean;
}

export interface RoomScheduleEntry {
  id: number;
  room_name: string;
  dept: string;
  date: string;        // "YYYY-MM-DD"
  start_time: number;  // unix timestamp
  end_time: number;    // unix timestamp
  notes: string;
}

export interface DeptRule {
  dept: string;
  priority_bonus: number;
  preferred_rooms: string[];
  color: string;
}

export interface TaskWithScore {
  task: SurgeryTask;
  score: number;
  is_overdue: boolean;
  deadline_elapsed_pct: number | null;
}

export interface ExtraComplianceResult {
  allowed: boolean;
  reason: string | null;
}

export interface StaffComplianceResult {
  staff_id: number;
  allowed: boolean;
  reason: string | null;
}

export interface SelfPayItem {
  id: number;
  name: string;
  price: number;
  notes: string;
}

export interface SyncTimestamps {
  last_push_at: number | null;
  last_pull_at: number | null;
}

export const URGENCY_WEIGHTS: Record<UrgencyLevel, number> = {
  Trauma: 100000,
  Level1: 80000,
  Level2: 60000,
  Level3: 40000,
  Normal: 10000,
};
