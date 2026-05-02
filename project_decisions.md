# 技術決策紀錄 (Architecture Decision Records)

本文件記錄重要技術決策，避免重複討論已決定的方案。

---

### ADR-001: SQLite 使用 runtime query，不使用 compile-time macro

**背景 (Context):**
sqlx 提供 `query!()` macro 可在編譯期驗證 SQL，但需要 `DATABASE_URL` 環境變數指向實際資料庫。Tauri 桌面應用的資料庫路徑由 `app_data_dir()` 在執行期動態決定，無法在編譯時確定。

**決策 (Decision):**
使用 `sqlx::query()` runtime API + 手動 `Row::get()` map_row 函數。`sqlx` features 不含 `macros`。

**理由 (Rationale):**
- 避免 CI 環境需要預先建立 SQLite 檔案
- 路徑動態、跨平台問題少
- 手動 map_row 雖然冗長，但型別明確、易於 debug

**否決方案 (Rejected Alternatives):**
- `query!()` macro：需在 build 前設定 `DATABASE_URL`，CI/CD 複雜度高
- SQLite embedded as compile-time asset：無法讓使用者資料持久化

---

### ADR-002: Google Apps Script 作為雲端同步後端

**背景 (Context):**
需要雲端備份/同步功能，但不想維護獨立伺服器基礎設施。目標使用者為醫院內部，Google Workspace 普及度高。

**決策 (Decision):**
使用 Google Apps Script (GAS) 部署為 Web App，以 Google Sheets 作為雲端資料儲存。Rust 端透過 `reqwest` 呼叫 GAS HTTP endpoints。

**理由 (Rationale):**
- 零伺服器成本，零維運負擔
- GAS URL 即為 endpoint，使用者自行部署掌控資料
- 同步模式為全量覆寫（syncAll），簡單可靠，不需要 conflict resolution

**否決方案 (Rejected Alternatives):**
- Firebase / Supabase：需要帳號管理、費用、網路策略
- 自建 REST API：需要 VPS + 維運
- 純本地模式：無法跨裝置備份

---

### ADR-003: Tauri invoke() 錯誤需明確 console.error 才進 DebugPanel

**背景 (Context):**
`useLogger` 攔截了 `window.fetch`，但 Tauri `invoke()` 走 IPC channel，不經過瀏覽器 fetch。當 Rust command 回傳 `Err(String)` 時，前端 `invoke()` 返回 rejected Promise，若只在 catch 設 `message.value` 而不呼叫 `console.error`，DebugPanel 完全看不到錯誤。

**決策 (Decision):**
所有 composable 的 `invoke()` catch block 凡需要被 DebugPanel 捕捉的錯誤，都必須補上 `console.error("[模組] 描述: ${e}")`。

**理由 (Rationale):**
- Logger 已有 console.error 攔截器，補一行即可，無需改動 Logger 架構
- GAS sync 錯誤（URL 未設定、網路逾時）是最常見的需要追查的錯誤

**否決方案 (Rejected Alternatives):**
- 攔截 `invoke` 函數本身：需要 monkey-patch `@tauri-apps/api/core`，版本升級易斷
- 讓所有 catch 都 re-throw 讓 `unhandledrejection` 捕捉：副作用難控制

---

### ADR-004: Room 與 RoomScheduleEntry 不設 FK 關聯

**背景 (Context):**
`rooms` 表儲存房間設定，`room_shifts` 表儲存月排班。兩者可透過 `room_name` 關聯，但 SQLite FK 預設關閉，且強制 FK 會導致先後匯入順序問題。

**決策 (Decision):**
`room_shifts` 只存 `room_name`（TEXT），不設 `room_id` foreign key。Timeline 顯示時以 room_name 字串匹配。匯入 xlsx 後若房間不存在，自動建立。

**理由 (Rationale):**
- 使用者可先匯入排班再補設定房間，不受順序限制
- 保持 schema 簡單，無需 JOIN

**否決方案 (Rejected Alternatives):**
- FK + cascade：需在匯入時保證房間先存在，UX 複雜

---

### ADR-005: Staff 類別以 staff_category 欄位區分，不拆表

**背景 (Context):**
人員分為「專責護理師（SA）」、「開刀房護理師」、「實習生」、「其他單位待訓練」四類，各有不同顯示樣式與業務含意。

**決策 (Decision):**
在 `staff` 表新增 `staff_category TEXT`（"sa" | "or_nurse" | "intern" | "cross_train"）與 `unit TEXT`（跨單位人員所屬單位）兩個欄位，不拆成多張子表。

**理由 (Rationale):**
- 四類人員共用絕大多數欄位（name/role/is_on_call/...）
- 勞基法合規計算（engine.rs）對所有類別邏輯一致
- 單表查詢簡單，前端 computed 依 category 分群顯示

**否決方案 (Rejected Alternatives):**
- 多型繼承（基礎表 + 子表）：過度設計，JOIN 複雜

---

### ADR-006: Excel 月排班在前端用 SheetJS 解析，不走 Tauri fs

**背景 (Context):**
匯入 .xlsx 月排班需要解析 Excel 格式。可選：前端 JS library 直接讀取 File 物件，或用 Tauri fs plugin 讀取後傳至 Rust 解析。

**決策 (Decision):**
使用 SheetJS (npm `xlsx`) 在前端透過 `FileReader.readAsArrayBuffer()` 解析，整理成 `RoomScheduleEntry[]` 後再透過 `invoke()` 存入 DB。

**理由 (Rationale):**
- FileReader 是 Web 標準 API，Tauri WebView 完整支援，不需要額外 capability
- SheetJS 生態成熟，支援中文欄名、日期序列號轉換
- Rust 端不需要引入 Excel 解析 crate（減少 binary 大小）

**否決方案 (Rejected Alternatives):**
- Rust calamine crate：需要用 Tauri fs 先把檔案讀進來，多一層 IPC
- CSV 轉換再匯入：醫院現有排班是 Excel，不要求使用者另存 CSV

---

### ADR-007: StaffAssignment（指派）與 StaffRosterEntry（班表）分兩張表

**背景 (Context):**
人員資料有兩種截然不同的用途：「今日哪個人被指派到哪間刀房擔任什麼角色」（即時拖拉操作）與「人員本月的班別排班（早班/晚班/休息）」（匯入 XLSX）。

**決策 (Decision):**
分成兩張獨立表：
- `staff_assignments (id, staff_name, room_name, date, role)` — 拖拉指派，即時新增/刪除
- `staff_roster (id, staff_name, date, shift_name)` — 月班表，整月覆寫匯入

**理由 (Rationale):**
- 兩者使用場景完全不同：assignments 是今日動態，roster 是靜態月曆
- assignments 需要 `add_one` / `remove_one` 即時操作；roster 只需整月覆寫
- 欄位結構不重疊：assignments 有 room_name/role，roster 有 shift_name
- 分表避免「班表 XLSX 匯入覆蓋了當日手動指派」的問題

**否決方案 (Rejected Alternatives):**
- 合併一張表 + type 欄位區分：查詢條件複雜，write 時容易誤蓋資料
- 只用 staff_assignments（把班別也存成 role）：語意混淆，無法支援休息/換班等狀態

---

### ADR-008: 房間排班 XLSX 以「週次樞紐表」格式解析，不要求使用者轉換格式

**背景 (Context):**
醫院現有 `2026手術時段.xlsx` 為樞紐表格式（12 分頁對應 12 個月，每分頁按週次分組，欄為星期一～日，列為刀房×AM/PM），不是平面 CSV 格式。

**決策 (Decision):**
在 `SettingsModal.vue` 的 `parseXlsx()` 中直接實作樞紐表解析：
偵測「第N週」header → 提取日期序列號 → 遍歷 AM/PM 列 → 產生 `RoomScheduleEntry[]`。

**理由 (Rationale):**
- 使用者直接使用醫院現有排班檔案，不需要手動轉換格式
- Excel 序列日期轉換（`serial - 25569` → UTC ms）是標準做法，SheetJS 的 raw 模式可直接取到數字
- AM/PM 時段固定（07:30～12:30 / 12:30～17:00），硬編碼於 `PERIOD_TIMES` 常數即可

**否決方案 (Rejected Alternatives):**
- 要求使用者另存為 CSV 或平面格式：增加使用者操作步驟，且每次更新都要重新轉換
- 通用格式自動偵測：複雜度過高，易誤判，不如針對已知格式直接解析
