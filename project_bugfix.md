# 除錯紀錄 (Bug Fix Log)

記錄修改三次（含）以上才解決的重要 Bug，以及具有平台特殊性的問題，作為未來避坑指南。

---

### BF-001: sqlx `query!` macro 無法在 Tauri 專案編譯

**問題描述:**
使用 `sqlx::query!("SELECT ...")` compile-time macro 時，`cargo check` 報錯：
`error: DATABASE_URL must be set to use query macros`

**嘗試過程:**
1. 嘗試設定 `DATABASE_URL=sqlite:./dev.db` 環境變數，仍報錯（路徑需為絕對路徑）
2. 嘗試建立 `.env` 檔案，cargo 未自動讀取
3. 嘗試 `sqlx prepare`，需要先連線到資料庫但 DB 路徑在執行期才確定

**根本原因:**
Tauri 應用的 SQLite 路徑由 `app.path().app_data_dir()` 在執行期動態決定，compile-time macro 無法知道路徑。

**最終解法:**
完全改用 `sqlx::query()` runtime API + 手動 `map_row()` 函數。從 `sqlx` features 中移除 `macros`：
```toml
sqlx = { version = "0.8", features = ["runtime-tokio-native-tls", "sqlite"] }
```

**牽扯檔案:** `src-tauri/Cargo.toml`、`src-tauri/src/db/*.rs`

---

### BF-002: `handle.manage()` 編譯錯誤 — method not found

**問題描述:**
在 `lib.rs` 的 `.setup()` 閉包中呼叫 `handle.manage(AppState { db: pool })`，報錯：
`error[E0599]: no method named 'manage' found for type 'AppHandle'`

**嘗試過程:**
1. 確認 `AppState` struct 存在且 public — 無效
2. 確認 `AppHandle` 型別正確 — 無效

**根本原因:**
`manage()` 是 `tauri::Manager` trait 的方法，需要明確 `use tauri::Manager;` 才能呼叫。

**最終解法:**
在 `lib.rs` 頂部加入：
```rust
use tauri::Manager;
```

**牽扯檔案:** `src-tauri/src/lib.rs`

---

### BF-003: Tailwind CSS v4 production build 報錯 `Cannot apply unknown utility class`

**問題描述:**
`npx vite build` 失敗：
`[@tailwindcss/vite:generate:build] Cannot apply unknown utility class 'form-input'`

**嘗試過程:**
1. 確認 `form-input` class 確實在 `@layer components` 內定義 — 存在但仍報錯
2. 改變 `@layer` 順序 — 無效
3. 在 dev 模式（`vite dev`）完全正常，只有 production build 報錯

**根本原因:**
Tailwind CSS v4 的 production build 不允許 `@apply` 引用同一 `@layer components` 內的其他自訂 class（`.form-select { @apply form-input ...; }` 這種寫法）。dev 模式因為 JIT 掃描順序不同而不報錯。

**最終解法:**
將 `.form-select` 的 `@apply form-input` 展開為全部原生 utility class，不在 `@apply` 中引用自訂 class：
```css
.form-select {
  @apply w-full bg-gray-700 border border-gray-600 rounded px-3 py-2
         text-sm text-gray-100 placeholder-gray-500 outline-none
         focus:border-blue-500 focus:ring-1 focus:ring-blue-500
         transition-colors appearance-none cursor-pointer;
}
```

**牽扯檔案:** `src/assets/main.css`

---

### BF-004: Debug Panel 無法捕捉 GAS sync 錯誤

**問題描述:**
GAS 同步（push/pull）失敗時，Sidebar 顯示錯誤訊息，但 Debug Panel（Ctrl+Shift+D）完全沒有任何 log 記錄。

**嘗試過程:**
1. 確認 `useLogger.initLogger()` 有在 `main.ts` 呼叫 — 正確
2. 確認 `window.fetch` 攔截器有啟動 — 有，但 GAS 請求完全沒走到這裡
3. 確認 `window.onerror` / `unhandledrejection` — 沒有觸發

**根本原因:**
GAS sync 在 Rust 端透過 `reqwest` 發出 HTTP 請求，前端透過 `invoke("sync_push")` 呼叫。`invoke()` 走 Tauri IPC channel，**完全不經過** `window.fetch`。`invoke()` rejected 的 Promise 被 `useSync.ts` 的 catch 攔截，不會冒泡到 `unhandledrejection`。

**最終解法:**
在 `useSync.ts` 的 catch block 補上 `console.error()`，由 Logger 的 `console.error` 攔截器接收：
```typescript
} catch (e) {
  status.value = "error";
  message.value = String(e);
  console.error(`[sync] push 失敗: ${e}`);  // ← 新增
  throw e;
}
```

**牽扯檔案:** `src/composables/useSync.ts`、`src/composables/useLogger.ts`

---

### BF-005: Windows 上 SQLite URL 路徑問題

**問題描述:**
`SqlitePoolOptions::new().connect(&url)` 在 Windows 上回傳錯誤，無法建立資料庫。

**根本原因:**
Windows 路徑使用反斜線（`C:\Users\...\ortriage.db`），但 SQLite URL 規格要求正斜線（`sqlite:C:/Users/.../ortriage.db`）。`to_string_lossy()` 在 Windows 保留反斜線。

**最終解法:**
```rust
let url = format!(
    "sqlite:{}?mode=rwc",
    db_path.to_string_lossy().replace('\\', "/")
);
```

**牽扯檔案:** `src-tauri/src/db/mod.rs`

---

### BF-006: Tauri 專案初始化失敗（非互動式終端）

**問題描述:**
在 Claude Code / CI 環境執行 `npm create tauri-app` 時報錯：`not a terminal`，無法互動式輸入選項。

**根本原因:**
`create-tauri-app` CLI 預設需要互動式 TTY 輸入專案選項。

**最終解法:**
在另一個目錄（`ORtable-init`）使用 `npx create-tauri-app@latest ORtable-init --template vue-ts --manager npm --yes` 或等效參數建立後，手動複製所需檔案至目標目錄。

**牽扯檔案:** 初始化流程（一次性，不影響後續開發）

---

### BF-007: Gemini 修改引入的多處 TypeScript 型別錯誤

**問題描述:**
Gemini 新增拖拉指派與 StaffPool 功能後，`npx vue-tsc --noEmit` 回報 8 個錯誤，涵蓋 5 個檔案。

**根本原因 / 修復清單:**

1. `useDragState.ts`：`dragType` 型別為 `"staff" | "task" | null`，未包含新加的 `"assignment"`。  
   → 改為 `"staff" | "task" | "assignment" | null`

2. `SettingsModal.vue` (`staffXlsxPreview`)：宣告為 `ref<StaffAssignment[]>` 但解析函式實際產生 `StaffRosterEntry[]`（有 `shift_name` 而非 `room_name`/`role`）。  
   → 改為 `ref<StaffRosterEntry[]>`；預覽表格欄位從「日期/房間/姓名/角色」改為「日期/姓名/班別」；呼叫由 `staffAssignmentsDb.replaceByMonth` 改為 `staffRosterDb.replaceByMonth`

3. `SettingsModal.vue`（未定義的 `roleTagClass`）：模板呼叫 `roleTagClass(a.role)` 但 script 中未定義此函式（因 preview 改為 roster 格式，直接顯示 `shift_name` 就不需要此函式）。  
   → 隨欄位調整一起移除

4. `TaskFormModal.vue`：`SurgeryTask` Phase 6 新增了 7 個欄位，但 `form` reactive 物件未補齊，導致型別錯誤。  
   → 補入 `seq_no: 0`, `gender: ""`, `age: 0`, `bed_no: ""`, `body_part: ""`, `anesthesia: ""`, `surgeon: ""`

5. `PatientCard.vue`：`const emit = defineEmits<...>()` 宣告了但模板使用 `$emit()` 直接呼叫，變數從未使用。  
   → 改為 `defineEmits<...>()` 不賦值

**附加修復：** `useDatabase.ts` 缺少 `useStaffRosterDb()` wrapper（對應後端 `get_staff_roster_by_date` / `replace_staff_roster_by_month`）。

**牽扯檔案:** `useDragState.ts`、`SettingsModal.vue`、`TaskFormModal.vue`、`PatientCard.vue`、`useDatabase.ts`

---

### BF-008: 房間月排班 XLSX 解析格式不符

**問題描述:**
匯入 `2026手術時段.xlsx` 後 DebugPanel 顯示：
`[XLSX] 解析失敗：未找到符合格式的資料。請確認 Excel 標題包含：日期、房間、科別、開始時間、結束時間。`

**嘗試過程:**
1. 確認 xlsx 檔案存在且可讀 — 正常
2. 以 Node.js 解析原始結構，發現格式與預期完全不同

**根本原因:**
舊版解析器假設「平面格式」（每列一條記錄，欄位：日期/房間/科別/開始時間/結束時間），但真實檔案為「**樞紐表格式**」：
- 12 個分頁（一月～十二月）
- 每分頁按週次（第 1 週、第 2 週…）分組
- 每週 header 列記錄星期一～日的 Excel 序列日期
- 每個刀房各佔兩列（AM / PM），欄值為科別代號

**最終解法:**
完整重寫 `parseXlsx()`：
- 遍歷所有分頁
- 偵測「第N週」開頭列 → 提取星期一～日 Excel 序列日期（`serial - 25569` 轉 Unix ms）
- 偵測 Col B = "AM" | "PM" → 對 Col C~I 逐格產生 `RoomScheduleEntry`
- AM = 07:30～12:30（`27000s`～`45000s` offset）；PM = 12:30～17:00（`45000s`～`61200s`）
- `xlsxMonth` 改為顯示「整年度」或涵蓋月份列表

解析結果（2026 年度）：1,486 筆、7 個月份（1～7 月有排班）、5 間刀房、12 個科別。

**牽扯檔案:** `src/components/SettingsModal.vue`（`parseXlsx` 函式、`PERIOD_TIMES` 常數、`excelSerialToDateStr` 輔助函式）
