專案名稱: Smart OR Triage (智慧手術排程看板) - Tauri 獨立運作版

1. 專案目標與背景

開發一套輕量、安全且可獨立運作 (免介接 HIS) 的桌面端應用程式，協助開刀房第一線醫護人員精準判斷病患進線 (入開刀房) 順序。系統需提供直覺的拖拉式 UI，並具備依據緊急程度與等待時間的動態排序演算法，同時涵蓋應對資源耗竭的「二線機制」與符合勞基法規範的「Extra 線機制」。

2. 系統架構設計 (技術棧)

前端 (Frontend): Vue 3 (Vite), Tailwind CSS v4 (@tailwindcss/vite 外掛), FontAwesome (圖示), Pinia (狀態管理)。

後端 (Backend): Rust (Tauri 2.0)。負責提供系統底層 API、檔案存取與高效能運算。

本地資料庫 (Local DB): SQLite (搭配 sqlx 0.8 套件)，用於儲存當日排程與人員設定。

DevOps & 更新:

GitHub Actions: 自動化編譯全平台版本 (Windows/macOS)。

Tauri Updater: 整合 tauri-plugin-updater 實作一鍵升級。

日誌系統 (Logging): 使用 tauri-plugin-log 統一收集前後端錯誤日誌至本地檔案，便於除錯。

3. UI/UX 設計規範

系統採 2:1 佈局，重點區塊如下：

左側 Sidebar: 功能選單 (匯入清單、建立急診病患、一鍵啟動二線機制、申請 Extra 線)。

區塊 1 (右側 1/3): 待排定區

今日上班成員 (含二線/Extra志願): 護理師池，區分「常規」、「二線待命」與「標註為可 Extra」的人員。

待排定病患列表: 顯示所有等待中病患小卡。卡片顏色依緊急程度區分，若即將超時呈閃爍紅框。

區塊 2 (左側 2/3): 房間總覽 (Timeline)

時間縱軸: 顯示 07:30 至隔日 07:30，每小時 60px。

房間標頭: 每個房間固定顯示「刷手」、「流動」、「助手」三個放置區，可無限堆疊人員卡片。

時間軸區域: 顯示各房間科別使用時段。隱藏的「備用刀房」或「Extra 房」在觸發機制時顯現並標示狀態。

4. 核心排序與業務邏輯 (Rule Engine)

採用「基礎權重 + 時間動態加分」機制：

二線機制與資源耗竭 (Emergency Override):

預判 Trauma/Level 1 無法於限制時間內進刀時發出警報，解鎖備用刀房與二線人員，賦予絕對優先權。

彈性擴充機制與勞基法防呆 (Extra Line Compliance):

適用時段：值班時段 (如 16:00 - 07:30)。

計薪標記：開啟的 Extra 房間與參與人員標記 extra_pay。

勞基法硬性阻擋邏輯 (Rust Backend 實作)：

條件一 (當日工時上限)：預估 Extra 結束時間 - today_shift_start <= 12 小時。

條件二 (班距間隔下限)：next_day_shift_start - 預估 Extra 結束時間 >= 12 小時。

若人員不符合上述任一條件，UI 端將強制反灰該人員卡片，禁止拖拉入房。

緊急級別限制 (最高優先):

Trauma (30m內): 基礎分 100000

Level 1 (2h內): 基礎分 80000

Level 2 (6h內): 基礎分 60000

Level 3 (24h內): 基礎分 40000

Normal (常規): 基礎分 10000

動態加權規則: 等待時間逼近極限時動態加分。

科別空房與特殊房匹配 (次高優先): 優先匹配專屬科別房間 (Block Time) 或急診專屬房。

預定時間優先: 優先比對 scheduled_at。

各科自訂優先規則 / 系統排入時間 (Tie-breaker): 依各科設定規則，無設定則依 created_at (FIFO)。

5. 核心資料模型 (Data Models)

UrgencyLevel (Enum): Trauma, Level1, Level2, Level3, Normal。

SurgeryTask (Struct): id, patient_name, chart_no, procedure, diagnosis, vs_note, dept, expected_room, urgency, scheduled_at, created_at, est_time_mins, status。

Staff (Struct):

id, name, role (R/Scrub/Circ/VS), type (doc/nur)

is_on_call (boolean), is_volunteer_extra (boolean)

today_shift_start (i64, 今日首班打卡/上班時間)

next_day_shift_start (i64, 明日表定上班時間)

RoomShift (Struct): room_name, period, dept, start_time, end_time, is_backup (boolean), is_emergency_priority (boolean), is_extra_line (boolean)。

6. 開發階段規劃 (Phases)

Phase 1: 專案初始化與 UI 基礎 ✅ 完成

建立 Tauri 2.0 + Vue 3 + TypeScript + Tailwind CSS v4 + Pinia 專案。
建立 2:1 主佈局 (App.vue)。
建立 DebugPanel 元件 (Ctrl+Shift+D 切換，useLogger composable)。
建立 Rust 資料模型 (models.rs) 與核心指令框架 (commands.rs)。
實作 get_sorted_tasks (優先權排序) 與 check_extra_compliance (勞基法防呆) Tauri command。

Phase 2: 核心排序引擎 (Rust) ✅ 完成

建立 engine.rs 純函數排序模組（5 個單元測試全過）。
Rule Engine 評分公式：urgency_base + dynamic_deadline_bonus + room_availability_bonus + dept_rule_bonus + scheduled_time_bonus，FIFO 做 tie-breaker。
dynamic_deadline_bonus：隨截止時間逼近，從 0 線性增長至 base_weight/2（Trauma 30min、Level1 2h、Level2 6h、Level3 24h）。
新增 DeptRule（科別自訂加分與偏好房間）。
新增 TaskWithScore（含 score、is_overdue、deadline_elapsed_pct）供 UI 呈現紅框閃爍。
Extra 線合規精算：精確顯示「當日工時 Xh YYm」與「班距僅 Xh YYm」。
新增 batch_check_extra_compliance 批次指令，一次回傳所有當班人員合規狀態。
前端：types/index.ts 補充所有新型別；useTaskEngine.ts composable 封裝四支 Tauri invoke 呼叫。

Phase 3: 資料持久化與狀態管理 ✅ 完成

SQLite 本地資料庫（sqlx 0.8，runtime 啟動時自動 migrate）。
Schema：surgery_tasks / staff / dept_rules / settings 四張表，含 AUTOINCREMENT PK 與 ON CONFLICT UPSERT。
DB 模組分層：db/tasks.rs、db/staff.rs、db/dept_rules.rs、db/settings.rs 各自負責 CRUD，replace_all 使用 transaction。
AppState（state.rs）持有 SqlitePool，透過 tauri::Manager::manage() 注入，lib.rs setup hook 以 block_on 同步初始化。
完整 CRUD commands：get/create/update/delete for tasks、staff；upsert/delete for dept_rules。
Settings：get_gas_url / set_gas_url 存於 SQLite settings 表。
雲端同步（Google Sheets + GAS）：
  gas/Code.gs：doGet（getTasks/getStaff/getDeptRules/ping）、doPost（syncAll 全量覆寫）、SyncLog 記錄每次同步。
  sync.rs：push（讀本地 → POST GAS）、pull（並行 GET 三資源 → replace_all）、reqwest 20s timeout。
  sync_push / sync_pull / get_sync_timestamps 三支 Tauri command。
前端：useDatabase.ts（useTasksDb/useStaffDb/useDeptRulesDb/useSettings）、useSync.ts（push/pull/formatTs/timestamps）。
Pinia stores 更新：tasks.ts / staff.ts 改為從 DB 載入，App.vue onMounted 同步初始化。

Phase 4: 進階功能與部署 ✅ 完成

Sidebar 全功能完善（Sidebar.vue 接收 backupActive prop，所有按鈕事件由 App.vue 統一管理）：
  匯入病患清單 → ImportModal：拖放/點擊選 CSV、自動解析欄位（支援中英文欄名）、預覽前 10 筆、略過空姓名列。
  新增急診病患 → TaskFormModal：色碼緊急程度選擇器（Trauma/Level1/Level2/Level3/Normal）、完整表單欄位、datetime-local 預排時間。
  啟動二線機制 → 紅色橫幅（附動態脈衝點）、解除按鈕、Sidebar ON 標籤，狀態由 App.vue 統一持有。
  申請 Extra 線 → ExtraLineModal：預設結束時間 +4h、即時 batch_check_extra_compliance、合規綠✓/不合規紅×（含精確原因）、選人＋指定刀房確認。
全域表單樣式（main.css @layer components：form-label / form-input / btn-primary / btn-secondary / btn-danger）。
tauri-plugin-updater 整合：useUpdater.ts（動態 import，silently ignore 未設定情境）、啟動後 3 秒背景檢查、有新版本時顯示頂部橫幅。
GitHub Actions CI/CD（.github/workflows/build.yml）：tag 觸發，Windows x86_64 + macOS ARM + macOS x86 三平台並行建置，使用 tauri-apps/tauri-action@v0，支援 TAURI_SIGNING_PRIVATE_KEY。

注意事項：
  Updater pubkey 需執行 `npm run tauri signer generate` 產生金鑰，將公鑰填入 tauri.conf.json plugins.updater.pubkey。
  GitHub Actions 需在 Repo Settings → Secrets 加入 TAURI_SIGNING_PRIVATE_KEY 與 TAURI_SIGNING_PRIVATE_KEY_PASSWORD。
  build.yml 中的 YOUR_ORG/ORtable 需替換為實際 GitHub 路徑。

7. 目錄結構

```
src/
  App.vue                   # 根元件，2:1 主佈局，Ctrl+Shift+D，modal 統一管理
  main.ts                   # createApp，初始化 Pinia，呼叫 initLogger()
  components/
    DebugPanel.vue           # 開發用 Log 浮動面板（Ctrl+Shift+D，單/雙擊複製）
    Sidebar.vue              # 左側功能選單（emit: import/addEmergency/toggleBackup/requestExtra/openSettings）
    TaskFormModal.vue        # 新增急診病患表單（色碼緊急程度選擇器）
    ImportModal.vue          # CSV + PDF 病患清單匯入（雙 tab；PDF 用 pdfjs-dist 解析手術紀錄表）
    ExtraLineModal.vue       # Extra 線申請（即時 batch_check_extra_compliance）
    SettingsModal.vue        # 設定（3 分頁：房間管理 / 人員管理 / 雲端設定）
    TimelinePanel.vue        # 房間總覽時間軸（07:30-07:30，60px/hr，科別色塊，現在線）
  composables/
    useLogger.ts             # singleton logger（攔截 console.error/warn, fetch, onerror）
    useDatabase.ts           # invoke wrappers：useTasksDb/useStaffDb/useRoomsDb/useRoomShiftsDb/useDeptRulesDb/useSettings
    useSync.ts               # GAS push/pull/timestamps（錯誤自動 console.error → DebugPanel）
    useUpdater.ts            # tauri-plugin-updater 背景更新檢查
    useTaskEngine.ts         # invoke 封裝：get_tasks_with_scores / batch_check_extra_compliance
  stores/
    tasks.ts                 # Pinia: SurgeryTask（load/add/edit/remove）
    staff.ts                 # Pinia: Staff（load/add/edit/remove）
    rooms.ts                 # Pinia: Room（load/add/edit/remove）
  types/
    index.ts                 # 全部 TS 型別與常數（UrgencyLevel/Staff/Room/RoomScheduleEntry/...）
  assets/
    main.css                 # Tailwind CSS v4 入口 + @layer components（form-label/form-input/btn-*）
src-tauri/
  src/
    lib.rs                   # Tauri builder：plugins + setup(block_on DB init) + invoke_handler(25 commands)
    main.rs                  # 程式進入點
    models.rs                # 全部 Rust struct（SurgeryTask/Staff/Room/RoomScheduleEntry/DeptRule/TaskWithScore/...）
    engine.rs                # Rule Engine：priority_score/sort_tasks/check_compliance/batch_compliance（5 單元測試）
    state.rs                 # AppState { db: SqlitePool }
    sync.rs                  # GAS HTTP push/pull（reqwest 0.12，tokio::try_join! 並行 GET）
    commands.rs              # 全部 Tauri commands（引擎 + Tasks/Staff/Rooms/RoomShifts/DeptRules/Settings/Sync）
  db/
    mod.rs                   # 連線初始化、migrate（6 張表）、ALTER TABLE 補欄位
    tasks.rs                 # surgery_tasks CRUD（runtime sqlx query，手動 map_row）
    staff.rs                 # staff CRUD（bool↔INTEGER，staff_category/unit 欄位）
    rooms.rs                 # rooms CRUD
    room_shifts.rs           # room_shifts：get_by_date / replace_by_month（transaction）
    dept_rules.rs            # dept_rules CRUD + upsert（preferred_rooms 為 JSON 字串）
    settings.rs              # settings key/value get/set
gas/
  Code.gs                    # GAS Web App（doGet: getTasks/getStaff/getDeptRules/ping；doPost: syncAll）
.github/
  workflows/
    build.yml                # tag 觸發，Windows x86_64 + macOS ARM + macOS x86 三平台並行建置
```

Phase 5: 房間管理、人員管理與 Timeline UI ✅ 完成

新增資料模型：
  Room { id, name, display_order, is_backup } — rooms 表
  RoomScheduleEntry { id, room_name, dept, date, start_time, end_time, notes } — room_shifts 表
  Staff 擴充 staff_category ("sa"|"or_nurse"|"intern"|"cross_train") 與 unit 欄位

DB 層：db/rooms.rs（CRUD）、db/room_shifts.rs（get_by_date / replace_by_month）
  migrate() 新增 rooms / room_shifts 表，並以 ALTER TABLE 為舊版 staff 表補欄位。

Tauri commands：get_all_rooms / create_room / update_room / delete_room、
  get_room_shifts_by_date / replace_room_shifts_by_month

前端：
  SettingsModal.vue — 4 分頁：房間管理（CRUD + xlsx 月排班匯入 auto-create rooms）、
    人員管理（SA/開刀房護理師/實習生/其他單位待訓練人員 CRUD + 批次新增）、
    科別管理（DeptRule CRUD + 色彩選擇器）、
    雲端設定（GAS URL）
  TimelinePanel.vue — 房間總覽時間軸（07:30-07:30，60px/hr、科別色塊、現在線、空房提示）
  Sidebar「設定」按鈕改為觸發 SettingsModal（emit openSettings）
  xlsx (SheetJS) 用於解析 .xlsx 月排班，支援中文欄名、自動補建房間

Phase 6: 人員指派、待排定區與 Schema 擴充 ✅ 完成

SurgeryTask 模型擴充（配合真實 PDF 排程欄位）：
  新增欄位：seq_no, gender, age, bed_no, body_part, anesthesia, surgeon
  db/mod.rs migrate() 以 ALTER TABLE 補欄位（冪等），向下相容

新增資料模型：
  StaffAssignment { id, staff_name, room_name, date, role } — staff_assignments 表（拖拉指派）
  StaffRosterEntry { id, staff_name, date, shift_name } — staff_roster 表（月班表）
  ShiftDefinition { name, start_time, end_time, is_on_call } — shift_definitions 表

DB 層：db/staff_assignments.rs（get_by_date / add_one / remove_one / replace_by_month）
       db/staff_roster.rs（get_by_date / replace_by_month）

Tauri commands：
  get_staff_assignments_by_date / replace_staff_assignments_by_month /
  add_staff_assignment / remove_staff_assignment（指派操作）
  get_staff_roster_by_date / replace_staff_roster_by_month（班表）

前端新元件：
  StaffPoolPanel.vue — 可分配人員面板（依類別分群、拖拉 / 右鍵指派至房間、右鍵兩步驟選房間+角色）
  PendingQueuePanel.vue — 待排定病患卡片列表（waiting 狀態）
  PatientCard.vue — 統一病患卡片元件（queue 模式 / timeline 模式）
  ContextMenu.vue — 可重用浮動右鍵選單（Teleport to body，跨元件使用）

前端新 Composable / Store：
  useDragState.ts — singleton 共享拖曳狀態（isDragging, dragType: "staff"|"task"|"assignment"|null）
  stores/assignments.ts — Pinia store：StaffAssignment（load/add/remove）
  stores/deptRules.ts — Pinia store：DeptRule（load/upsert/remove，從 SettingsModal 抽離）

SettingsModal.vue 擴充：
  人員月排班（Staff Roster）XLSX 匯入 — 解析「日期/姓名/班別」格式存入 staff_roster 表
  XLSX 解析器重寫（房間月排班）：支援真實樞紐表格式（週次/AM/PM/值班/星期一~日），
    每分頁一個月份，全年 12 分頁一次匯入，AM=07:30~12:30、PM=12:30~15:50、值班=15:50~隔日07:30

useDatabase.ts 擴充：
  useStaffAssignmentsDb()：invoke wrapper for 4 commands
  useStaffRosterDb()：invoke wrapper for 2 commands
  useTasksDb().batchCreate()：invoke wrapper for batch_create_tasks

Phase 8: 病患狀態管理、詳細資料 Modal、科別房間分配、病人清單視圖 ✅ 完成

新增 TaskStatus 型別（types/index.ts）：
  waiting → scheduled → called（已叫刀）→ in_surgery（手術中）→ recovery（恢復室）
  TASK_STATUS_LABELS / TASK_STATUS_COLORS 常數供各元件共用

新增元件：
  PatientDetailModal.vue — 點擊病患卡片開啟；顯示全部欄位、狀態快速切換列、
    inline 編輯模式（12 欄表單）、刪除（含確認 overlay）；Teleport to body
  DeptRoomModal.vue — 科別房間分配；日期選擇器（計算星期）、CRUD 表格
    （inline 編輯列 + 新增列）、儲存呼叫 replace_room_shifts_by_date 僅覆寫當日
  PatientListPanel.vue — 本日病人清單全頁視圖；搜尋 + 狀態篩選、點整列開啟
    PatientDetailModal、刪除確認彈窗；App.vue view='patient_list' 切換顯示

Sidebar.vue 新按鈕：
  「科別房間分配」(teal) → emit open-dept-room → App.vue modal='dept_room'
  「本日病人清單」(cyan) → emit open-patient-list → App.vue view='patient_list'

App.vue：
  modal 型別擴充加 'dept_room'；新增 view ref ('board'|'patient_list')；
  v-if/v-else-if 切換 Board / PatientListPanel；掛載 DeptRoomModal

PatientCard.vue（timeline 模式）：
  狀態標籤 badge：叫（amber）/ 進（green/pulse）/ 完（purple）

TimelinePanel.vue：
  病患卡加 @click → detailTask（PatientDetailModal）
  右鍵選單加：已叫刀 / 手術中（進線）/ 恢復室 三個快速狀態切換按鈕（setTaskStatus()）
  scheduledTasksForRoom：今天顯示所有非 waiting；歷史日期以 scheduled_at 落點篩選

PendingQueuePanel.vue：
  病患卡加 @click → detailTask（PatientDetailModal）

新增後端：
  db/room_shifts.rs → replace_by_date()（DELETE + INSERT for single date in tx）
  commands.rs → replace_room_shifts_by_date command
  lib.rs → 註冊新 command（現共 35 支）
  useDatabase.ts → useRoomShiftsDb().replaceByDate()

Phase 7: PDF 匯入與日期選擇器 ✅ 完成

ImportModal.vue 擴充（CSV + PDF 雙模式）：
  PDF 排程表 tab — pdfjs-dist v5 解析 HIS 匯出的「手術紀錄表」橫式 PDF
  解析策略：text item 位置分群 → 行列重建 → 17 欄對應 → rowToTask()
  民國年轉換（ROC + 1911）、預起時間組成 scheduled_at unix timestamp
  房號 WT → status="waiting"；其餘房號 → status="scheduled"
  批次匯入：batch_create_tasks（transaction，單次 invoke）
  預覽分頁：全部 / 已排 / 待排；顯示房號/序號/姓名/科別/手術者/麻醉/預起/狀態

新增 Tauri command：
  batch_create_tasks(tasks: SurgeryTask[]) — 批次 INSERT，使用 transaction

TimelinePanel.vue 日期選擇器：
  Toolbar 日期改為 <input type="date">，[color-scheme:dark] 深色日曆彈窗
  切換日期重新載入 room_shifts + assignments；非今天顯示「今天」跳回按鈕
  nowY（現在線）邏輯不變：查看非今天時自動隱藏

7. 目錄結構

```
src/
  App.vue                   # 根元件，2:1 主佈局，Ctrl+Shift+D，modal 統一管理
  main.ts                   # createApp，初始化 Pinia，呼叫 initLogger()
  components/
    DebugPanel.vue           # 開發用 Log 浮動面板（Ctrl+Shift+D，單/雙擊複製）
    Sidebar.vue              # 左側功能選單（emit: import/addEmergency/toggleBackup/requestExtra/openSettings）
    TaskFormModal.vue        # 新增急診病患表單（色碼緊急程度選擇器）
    ImportModal.vue          # CSV 病患清單匯入（拖放 + FileReader + 中英文欄名映射）
    ExtraLineModal.vue       # Extra 線申請（即時 batch_check_extra_compliance）
    SettingsModal.vue        # 設定（4 分頁：房間管理 / 人員管理 / 科別管理 / 雲端設定）
    TimelinePanel.vue        # 房間總覽時間軸（07:30-07:30，60px/hr，科別色塊，現在線）
    StaffPoolPanel.vue       # 可分配人員面板（依類別分群，拖拉/右鍵指派）
    PendingQueuePanel.vue    # 待排定病患卡片（waiting 狀態列表）
    PatientCard.vue          # 統一病患卡片（queue / timeline 兩種模式）
    ContextMenu.vue          # 可重用浮動右鍵選單（Teleport to body）
  composables/
    useLogger.ts             # singleton logger（攔截 console.error/warn, fetch, onerror）
    useDatabase.ts           # invoke wrappers：useTasksDb/useStaffDb/useRoomsDb/useRoomShiftsDb/
                             #   useDeptRulesDb/useStaffAssignmentsDb/useStaffRosterDb/useSettings
    useSync.ts               # GAS push/pull/timestamps（錯誤自動 console.error → DebugPanel）
    useUpdater.ts            # tauri-plugin-updater 背景更新檢查
    useTaskEngine.ts         # invoke 封裝：get_tasks_with_scores / batch_check_extra_compliance
    useDragState.ts          # singleton：isDragging / dragType（"staff"|"task"|"assignment"|null）
  stores/
    tasks.ts                 # Pinia: SurgeryTask（load/add/edit/remove）
    staff.ts                 # Pinia: Staff（load/add/edit/remove）
    rooms.ts                 # Pinia: Room（load/add/edit/remove）
    assignments.ts           # Pinia: StaffAssignment（load/add/remove）
    deptRules.ts             # Pinia: DeptRule（load/upsert/remove）
  types/
    index.ts                 # 全部 TS 型別與常數
  assets/
    main.css                 # Tailwind CSS v4 入口 + @layer components（form-label/form-input/btn-*）
src-tauri/
  src/
    lib.rs                   # Tauri builder：plugins + setup(block_on DB init) + invoke_handler(33 commands)
    main.rs                  # 程式進入點
    models.rs                # 全部 Rust struct（含 StaffAssignment/StaffRosterEntry/ShiftDefinition）
    engine.rs                # Rule Engine：priority_score/sort_tasks/check_compliance（5 單元測試）
    state.rs                 # AppState { db: SqlitePool }
    sync.rs                  # GAS HTTP push/pull（reqwest 0.12，tokio::try_join! 並行 GET）
    commands.rs              # 全部 Tauri commands（33 支）
  db/
    mod.rs                   # 連線初始化、migrate（9 張表）、ALTER TABLE 補欄位
    tasks.rs                 # surgery_tasks CRUD
    staff.rs                 # staff CRUD（bool↔INTEGER，staff_category/unit 欄位）
    rooms.rs                 # rooms CRUD
    room_shifts.rs           # room_shifts：get_by_date / replace_by_month（transaction）
    dept_rules.rs            # dept_rules CRUD + upsert（preferred_rooms 為 JSON 字串）
    settings.rs              # settings key/value get/set
    staff_assignments.rs     # staff_assignments：get_by_date / add_one / remove_one / replace_by_month
    staff_roster.rs          # staff_roster：get_by_date / replace_by_month
```

8. 已知問題 / 待辦

- 初次使用需手動至「設定 → 人員管理」新增人員，才能使用可分配人員面板
- PendingQueuePanel 完整的拖拉至 Timeline 排班尚待驗證
- staff_roster XLSX 匯入的班別與勞基法防呆計算尚未整合（today_shift_start 仍為手動設定）
- 房間月排班 XLSX 目前只解析樞紐表格式（週次/時段/星期），不支援其他版型
