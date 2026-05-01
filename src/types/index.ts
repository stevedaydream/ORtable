export type UrgencyLevel = "Trauma" | "Level1" | "Level2" | "Level3" | "Normal";
export type StaffRole = "R" | "Scrub" | "Circ" | "VS";
export type StaffType = "doc" | "nur";
export type StaffCategory = "sa" | "or_nurse" | "intern" | "cross_train";

export const STAFF_CATEGORY_LABELS: Record<StaffCategory, string> = {
  sa:          "專責護理師",
  or_nurse:    "開刀房護理師",
  intern:      "實習生",
  cross_train: "其他單位待訓練",
};

export interface SurgeryTask {
  id: number;
  patient_name: string;
  chart_no: string;
  procedure: string;
  diagnosis: string;
  vs_note: string;
  dept: string;
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
