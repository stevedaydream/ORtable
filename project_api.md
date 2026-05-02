# API 文件 (API Reference)

## 外部 API

### Google Apps Script (GAS) Web App

部署後的 URL 儲存於 SQLite `settings` 表（key: `gas_url`）。

#### GET endpoints（Rust `sync::pull` 並行呼叫）

| URL 參數 | 回傳 | 說明 |
|----------|------|------|
| `?action=getTasks` | `{ tasks: SurgeryTask[] }` | 全部手術任務 |
| `?action=getStaff` | `{ staff: Staff[] }` | 全部人員 |
| `?action=getDeptRules` | `{ deptRules: DeptRule[] }` | 全部科別規則 |
| `?action=ping` | `{ ok: true }` | 連線測試 |

#### POST endpoint（Rust `sync::push` 呼叫）

```json
{
  "action": "syncAll",
  "tasks": [...],
  "staff": [...],
  "dept_rules": [...]
}
```
回傳：`{ ok: true, synced_at: <unix_timestamp> }`

---

## 內部介面（Tauri Commands）

### Rule Engine（純計算，無 state）

| Command | 參數 | 回傳 |
|---------|------|------|
| `get_tasks_with_scores` | `tasks: SurgeryTask[], dept_rules: DeptRule[], available_rooms: string[]` | `TaskWithScore[]` |
| `get_sorted_tasks` | `tasks: SurgeryTask[]` | `SurgeryTask[]` |
| `check_extra_compliance` | `today_shift_start: i64, next_day_shift_start: i64, estimated_extra_end: i64` | `ExtraComplianceResult` |
| `batch_check_extra_compliance` | `staff_list: Staff[], estimated_extra_end: i64` | `StaffComplianceResult[]` |

### Surgery Tasks CRUD

| Command | 參數 | 回傳 |
|---------|------|------|
| `get_all_tasks` | — | `SurgeryTask[]` |
| `create_task` | `task: SurgeryTask` | `SurgeryTask`（含 DB 產生的 id） |
| `update_task` | `task: SurgeryTask` | `SurgeryTask` |
| `delete_task` | `id: i64` | `void` |

### Staff CRUD

| Command | 參數 | 回傳 |
|---------|------|------|
| `get_all_staff` | — | `Staff[]` |
| `create_staff` | `staff: Staff` | `Staff` |
| `update_staff` | `staff: Staff` | `Staff` |
| `delete_staff` | `id: i64` | `void` |

### Rooms CRUD

| Command | 參數 | 回傳 |
|---------|------|------|
| `get_all_rooms` | — | `Room[]` |
| `create_room` | `room: Room` | `Room` |
| `update_room` | `room: Room` | `Room` |
| `delete_room` | `id: i64` | `void` |

### Room Shifts

| Command | 參數 | 回傳 |
|---------|------|------|
| `get_room_shifts_by_date` | `date: string`（"YYYY-MM-DD"） | `RoomScheduleEntry[]` |
| `replace_room_shifts_by_month` | `month: string`（忽略，以 entries 日期自動判斷）, `entries: RoomScheduleEntry[]` | `void` |

### Staff Assignments（拖拉指派）

| Command | 參數 | 回傳 |
|---------|------|------|
| `get_staff_assignments_by_date` | `date: string`（"YYYY-MM-DD"） | `StaffAssignment[]` |
| `replace_staff_assignments_by_month` | `month: string`, `entries: StaffAssignment[]` | `void` |
| `add_staff_assignment` | `entry: StaffAssignment` | `StaffAssignment`（含 DB id） |
| `remove_staff_assignment` | `id: i64` | `void` |

### Staff Roster（月班表）

| Command | 參數 | 回傳 |
|---------|------|------|
| `get_staff_roster_by_date` | `date: string`（"YYYY-MM-DD"） | `StaffRosterEntry[]` |
| `replace_staff_roster_by_month` | `month: string`（忽略，以 entries 日期自動判斷）, `entries: StaffRosterEntry[]` | `void` |

### Dept Rules

| Command | 參數 | 回傳 |
|---------|------|------|
| `get_dept_rules` | — | `DeptRule[]` |
| `upsert_dept_rule` | `rule: DeptRule` | `DeptRule` |
| `delete_dept_rule` | `dept: string` | `void` |

### Settings

| Command | 參數 | 回傳 |
|---------|------|------|
| `get_gas_url` | — | `string \| null` |
| `set_gas_url` | `url: string` | `void` |

### Cloud Sync

| Command | 參數 | 回傳 |
|---------|------|------|
| `sync_push` | — | `string`（成功訊息） |
| `sync_pull` | — | `string`（成功訊息） |
| `get_sync_timestamps` | — | `{ last_push_at: number \| null, last_pull_at: number \| null }` |

---

## 資料格式備忘

### SurgeryTask 完整欄位（Phase 6 擴充後）
```typescript
interface SurgeryTask {
  id: number;
  seq_no: number;       // PDF 序號
  patient_name: string;
  gender: string;       // "男" | "女"
  age: number;
  chart_no: string;
  bed_no: string;       // 病床號（"ICU02", "門診" 等）
  dept: string;
  diagnosis: string;
  body_part: string;    // PDF 部位欄位
  procedure: string;    // 術式
  anesthesia: string;   // "G" | "E" | "IVG" 等
  surgeon: string;      // 主刀醫師
  vs_note: string;
  expected_room: string;
  urgency: UrgencyLevel;
  scheduled_at: number | null;  // unix timestamp
  created_at: number;
  est_time_mins: number;
  status: string;       // "waiting" | "scheduled" | "completed"
}
```

### StaffAssignment
```typescript
interface StaffAssignment {
  id: number;
  staff_name: string;
  room_name: string;
  date: string;   // "YYYY-MM-DD"
  role: string;   // "Scrub" | "Circ" | "SA" | "R" | "VS"
}
```

### StaffRosterEntry
```typescript
interface StaffRosterEntry {
  id: number;
  staff_name: string;
  date: string;       // "YYYY-MM-DD"
  shift_name: string; // e.g., "D"(日班), "N1", "OFF"
}
```

### 時間戳
- 所有時間欄位均為 **Unix timestamp（秒）**，`i64` / `number`
- `scheduled_at`：可為 `null`（未指定預排時間）
- `today_shift_start` / `next_day_shift_start`：今日上班時間 / 明日表定上班時間，用於勞基法計算

### 緊急程度（UrgencyLevel）
Rust enum / TS union string：`"Trauma"` | `"Level1"` | `"Level2"` | `"Level3"` | `"Normal"`

在 SQLite 中以 TEXT 儲存；`str_to_urgency()` / `urgency_str()` 負責轉換。

### 人員類別（StaffCategory）
`"sa"` | `"or_nurse"` | `"intern"` | `"cross_train"`

`cross_train` 時 `unit` 欄位填所屬單位名稱，其他類別 `unit` 為空字串。

### Bool 在 SQLite
一律 INTEGER (0/1)。前端 TS 型別為 `boolean`，Rust 讀取用 `r.get::<i64, _>("col") != 0`。

### DeptRule.preferred_rooms
SQLite 儲存為 JSON 字串（如 `["OR1","OR3"]`），Rust 端 `serde_json::from_str` / `to_string` 轉換。

### RoomScheduleEntry.date
格式固定為 `"YYYY-MM-DD"`（字串）。`replace_by_month` 用 `date LIKE 'YYYY-MM%'` 清除整月資料。

### TaskWithScore
```typescript
{
  task: SurgeryTask,
  score: number,        // 最終優先分數（越高越優先）
  is_overdue: boolean,  // 已超過截止時間
  deadline_elapsed_pct: number | null  // 0–100，Normal urgency 為 null
}
```

### ExtraComplianceResult / StaffComplianceResult
```typescript
{ allowed: boolean, reason: string | null }  // reason 在 allowed=false 時說明不合規原因
{ staff_id: number, allowed: boolean, reason: string | null }
```
