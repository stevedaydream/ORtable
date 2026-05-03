# 功能優化建議 (Improvement Proposals)

記錄待規劃實作的功能優化方向，依優先順序排列。

---

## P1｜匯入時病歷號重複 / 同日兩科手術偵測

### 背景
HIS 匯出的 PDF 可能包含同一病患在同一天由不同科別（不同手術者）進行的手術，
目前匯入時不會偵測，導致同病患兩張卡片在 Timeline 上毫無關聯。

### 判斷邏輯

```
匯入的每筆資料
  ├─ 與【既有排程】比對 chart_no
  │     ├─ surgeon 相同 → 🟡 可能重複匯入，警告
  │     └─ surgeon 不同 → 🔵 同患者兩科手術（合併/同時進行）
  │
  └─ 與【本批次內】其他資料比對 chart_no
        ├─ surgeon 相同 → 🟡 批次內重複列
        └─ surgeon 不同 → 🔵 同患者兩科（批次內同時匯入）
```

### UI 設計（ImportModal 預覽表格）

在預覽列最左側新增「狀態」badge 欄：

| Badge | 顏色 | 意義 |
|-------|------|------|
| `重複` | 橘 | 相同病歷號 + 相同手術者，疑似重複匯入 |
| `兩科` | 藍 | 相同病歷號 + 不同手術者，同日跨科手術 |

「兩科」的額外處理：
- 匯入後自動在 `vs_note` 加註 `[合刀: {另一科術者}]`
- Timeline 兩張病患卡顯示連結圖示 🔗，hover 顯示「與 {科別} 同時進行」
- `SurgeryTask` 新增 `linked_task_id: number | null` 欄位建立雙向關聯

### 需修改的檔案
- `src/components/ImportModal.vue`：預覽階段加入比對邏輯與 badge 欄
- `src/types/index.ts`：`SurgeryTask` 新增 `linked_task_id` 欄位
- `src-tauri/src/models.rs`：對應 Rust struct 補欄位
- `src-tauri/src/db/mod.rs`：migrate 補 `linked_task_id` 欄位（ALTER TABLE）
- `src/components/PatientCard.vue`：timeline 模式顯示 🔗 連結 badge

---

## P2｜預排明日手術

### 背景
協調者需在今天就完成明天的手術排程（房間指派、人員安排），
目前 TimelinePanel 雖可切換日期查看，但匯入 PDF 時沒有「目標日期」選項，
且 PendingQueuePanel 不區分日期，明日待排病患與今日混在一起。

### 架構設計

```
ImportModal 新增「目標日期」選擇器（預設今天）
     ↓
匯入時所有 scheduled_at 對應目標日期
     ↓
App.vue 將 selectedDate 提升為共用 state
     ↓
TimelinePanel + PendingQueuePanel 同步顯示同一日期
```

### 需修改的檔案

**`src/components/ImportModal.vue`**
- PDF / CSV Tab 頂部新增日期選擇器
- 解析完成後將 `scheduled_at` 設為目標日期的對應時間（而非強制 today）

**`src/App.vue`**
- 新增 `selectedDate: ref<string>` 共用狀態（格式 `YYYY-MM-DD`）
- 傳給 TimelinePanel（`:selected-date`）與 PendingQueuePanel（`:selected-date`）

**`src/components/PendingQueuePanel.vue`**
- 接收 `selectedDate` prop
- `waitingTasks` 改為日期感知篩選：
  - `status === "waiting"` 且 `scheduled_at` 屬於 selectedDate（或 scheduled_at 為 null 歸今日）

**`src/components/TimelinePanel.vue`**
- 將內部 `todayStr` 提升為 emit，同步回 App.vue 的 selectedDate

---

## P3｜急診 / 加排手術快速房間推薦

### 背景
急診病患或臨時加排手術需要快速決定排入哪個刀房，
目前需要護理師憑經驗判斷，且需開啟多個畫面才能掌握全局。
本功能將所有決策條件按鈕化，系統自動計算並輸出推薦排名。

### 觸發入口
Sidebar 新增按鈕「快速派房」（或從「新增急診病患」流程整合）

### UI 流程（分層點選）

```
【第 1 層】緊急程度
  [Trauma 30分] [Level1 2小時] [Level2 6小時] [Level3 24小時] [Regular]

【第 2 層】科別（從 DeptRule 動態產生按鈕）
  [骨科] [神外] [心臟外] [泌尿] [婦科] ...

【第 3 層】麻醉方式
  [全身麻醉] [半身麻醉] [局部麻醉]

【第 4 層】預估時長
  [< 1小時] [1~2小時] [2~4小時] [> 4小時]

    ↓ 每層選完後即時更新推薦

【推薦結果】
  ┌──────────────────────────────────────────┐
  │ ✅ 首選：OR2 — 骨科（現空）              │
  │    預計可開始：立即                       │
  │    人員就位：刷手○ 流動○ 助手●           │
  ├──────────────────────────────────────────┤
  │ ⬜ 備選：OR5 — 急診備用房                │
  │    預計可開始：12:15（現有刀約 45 分鐘）  │
  ├──────────────────────────────────────────┤
  │ ⚠️  不建議：OR1 — 科別不符（心臟房）     │
  └──────────────────────────────────────────┘

  [取消]  [確認排入 OR2，立即建立急診單]
```

### 推薦演算法（Rust engine 擴充）

新增 command：`get_room_recommendation(urgency, dept, anesthesia, est_mins)`

計分規則：

| 條件 | 加分 |
|------|------|
| 科別符合 DeptRule preferred_rooms | +5000 |
| 房間目前為空（無 in_surgery / called 病患）| +4000 |
| 房間預計空出時間 < urgency 截止時間 | +3000 |
| 急診專屬房 is_emergency_priority | +2000 |
| 人員已就位（有 Scrub + Circ 指派）| +1000 |
| 房間為備用刀房（is_backup）| -1000 |

截止時間計算：
```
urgency 截止時間 = now + (Trauma: 30分 / L1: 2hr / L2: 6hr / L3: 24hr)
房間預計空出 = max(in_surgery task 的 scheduled_at + est_time_mins)
若 房間預計空出 > 截止時間 → 標記紅色警示
```

### 確認後自動執行
1. 建立 `SurgeryTask`（urgency / dept / expected_room / scheduled_at 已預填，status=`scheduled`）
2. 跳出 `TaskFormModal` 補填病患姓名、病歷號、手術名稱
3. 病患直接出現在 Timeline 目標房間

### 需新增 / 修改的檔案
- `src/components/QuickAssignModal.vue`：分層按鈕 UI + 結果顯示（新增）
- `src/components/Sidebar.vue`：新增「快速派房」按鈕與 emit
- `src/App.vue`：掛載 QuickAssignModal，modal 型別擴充
- `src-tauri/src/engine.rs`：新增 `get_room_recommendation()` 純函數
- `src-tauri/src/commands.rs`：新增對應 Tauri command
- `src-tauri/src/lib.rs`：註冊新 command

---

## P4｜整體 UX 優化（小型改善）

### 4-1 快速開始引導

啟動時若偵測到今日無任何任務，主動顯示「今日快速設定」引導：
1. 匯入 PDF
2. 科別房間分配
3. 確認人員出勤

**檔案**：`src/App.vue`（onMounted 後判斷，顯示引導 banner 或 modal）

### 4-2 匯入差異比對

若今日已有資料再次匯入 PDF，自動比對新舊版本差異，
只顯示「新增 / 變更 / 取消」項目，避免覆蓋已做的排班調整。

**檔案**：`src/components/ImportModal.vue`（匯入前與 tasksStore 比對）

### 4-3 人員出勤快速確認

啟動時顯示今日班表人員清單，打勾確認出勤，
同步自動設定 `today_shift_start`，解決勞基法防呆目前需手動設定的問題。

**前提**：需先完成 staff_roster XLSX 與 ShiftDefinition 班表時間的整合。

**檔案**：
- `src/components/AttendanceModal.vue`（新增）
- `src-tauri/src/commands.rs`：批次更新 `today_shift_start`

### 4-4 Timeline 房間摘要列

房間標頭下方顯示今日刀數與目前狀態：
- 「3 刀｜進行中」（綠色）
- 「5 刀｜空刀」（灰色）

**檔案**：`src/components/TimelinePanel.vue`（房間標頭計算 `scheduledTasksForRoom` 長度與 status）

---

## 實作優先順序總覽

| 優先 | 功能 | 複雜度 | 效益 |
|------|------|--------|------|
| 🔴 P1 | 病歷號重複 / 兩科手術偵測 | 低（前端邏輯） | 高（防止資料錯誤） |
| 🟠 P2 | 預排明日手術 | 中（架構調整） | 高（日常必要） |
| 🟡 P3 | 快速房間推薦 | 高（Rust engine）| 高（核心差異化） |
| 🟢 P4 | UX 小型改善 | 低～中 | 中（使用體驗） |
