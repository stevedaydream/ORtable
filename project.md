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
  App.vue                  # 根元件，2:1 主佈局，監聽 Ctrl+Shift+D
  main.ts                  # createApp，初始化 Pinia
  components/
    DebugPanel.vue          # 開發用 Log 浮動面板
  composables/
    useLogger.ts            # singleton logger，攔截全域錯誤
    useDatabase.ts          # DB CRUD wrappers (useTasksDb/useStaffDb/useDeptRulesDb/useSettings)
    useSync.ts              # GAS push/pull/timestamps
  stores/
    tasks.ts                # Pinia: SurgeryTask 狀態
    staff.ts                # Pinia: Staff 狀態
  types/
    index.ts                # TypeScript 型別定義與常數
  assets/
    main.css                # Tailwind CSS 入口
src-tauri/
  src/
    lib.rs                  # Tauri builder，載入外掛與 commands
    main.rs                 # 程式進入點
    models.rs               # Rust 資料結構 (+ DeptRule, TaskWithScore, StaffComplianceResult)
    engine.rs               # Rule Engine：priority_score / sort_tasks / batch_compliance
    state.rs                # AppState { db: SqlitePool }
    sync.rs                 # GAS HTTP push/pull（reqwest）
    commands.rs             # 全部 Tauri commands（引擎 + CRUD + sync）
  db/
    mod.rs                  # 連線初始化、migrate（4 張表）
    tasks.rs                # surgery_tasks CRUD
    staff.rs                # staff CRUD
    dept_rules.rs           # dept_rules CRUD + upsert
    settings.rs             # settings key/value get/set
gas/
  Code.gs                   # GAS Web App 腳本（部署至 Google Apps Script）
  composables/
    useTaskEngine.ts        # 封裝 invoke：loadTasks / checkStaffCompliance / checkSingleCompliance
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
  SettingsModal.vue — 3 分頁：房間管理（CRUD + xlsx 月排班匯入 auto-create rooms）、
    人員管理（SA/開刀房護理師/實習生/其他單位待訓練人員 CRUD）、
    雲端設定（GAS URL，從 Sidebar 移入）
  TimelinePanel.vue — 房間總覽時間軸（07:30-07:30，60px/hr，科別色塊、現在線、空房提示）
  Sidebar「設定」按鈕改為觸發 SettingsModal（emit openSettings）
  xlsx (SheetJS) 用於解析 .xlsx 月排班，支援中文欄名、自動補建房間

8. 已知問題 / 待辦

- 待排定區 TaskQueue UI 尚未實作（顯示 waiting 病患卡片、拖拉分配）
- Timeline 拖拉人員指派（刷手/流動/助手至房間標頭）尚未實作
