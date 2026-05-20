# 專案慣例 (Project Conventions)

## 技術棧快速索引

| 層次 | 技術 | 備注 |
|------|------|------|
| 前端框架 | Vue 3 (Composition API, `<script setup>`) | Pinia 狀態管理 |
| 建構工具 | Vite + `@tailwindcss/vite` 外掛 | Tailwind CSS v4，不用 PostCSS |
| 樣式 | Tailwind CSS v4 | `@import "tailwindcss"` 在 main.css |
| 圖示 | FontAwesome Free 6 | `fa-solid`、`fa-regular` prefix |
| 桌面框架 | Tauri 2.0 | Rust backend，WebView frontend |
| 後端語言 | Rust (edition 2021) | sqlx 0.8、reqwest 0.12、tokio 1 |
| 本地資料庫 | SQLite via sqlx 0.8 | runtime queries（無 compile-time macros） |
| Excel 解析 | SheetJS (xlsx npm) | 前端解析，不走 Tauri fs |
| 雲端同步 | Google Apps Script (GAS) | reqwest 從 Rust 端呼叫 |

---

## 檔案結構慣例

- **Vue 元件**：`src/components/XxxModal.vue`、`src/components/XxxPanel.vue`
- **Composables**：`src/composables/useXxx.ts`，camelCase，以 `use` 開頭
- **Pinia stores**：`src/stores/xxx.ts`，`defineStore("xxx", () => { ... })` setup 語法
- **TypeScript 型別**：全集中於 `src/types/index.ts`，不在 `.vue` 內定義 interface
- **Rust modules**：`src-tauri/src/db/` 一個模組一張表，`commands.rs` 集中所有 command handler

---

## 資料庫操作慣例

### sqlx runtime query 模式（重要）
本專案**不使用** `sqlx::query!()` compile-time macro（需要 `DATABASE_URL` 環境變數）。
一律使用 `sqlx::query()` + 手動 `Row::get()`：

```rust
use sqlx::Row;

fn map_row(r: &sqlx::sqlite::SqliteRow) -> MyStruct {
    MyStruct {
        id:   r.get("id"),
        name: r.get("name"),
        flag: r.get::<i64, _>("flag") != 0,   // bool 用 i64 存
    }
}
```

### Boolean 欄位
SQLite 不支援 BOOL，一律存 INTEGER (0/1)：
- 讀取：`r.get::<i64, _>("col") != 0`
- 寫入：`s.flag as i64`

### Transaction
批次覆寫操作（replace_all / replace_by_month）使用 transaction：
```rust
let mut tx = pool.begin().await?;
// ... operations on &mut *tx ...
tx.commit().await?
```

### Migration 策略
- `CREATE TABLE IF NOT EXISTS` 建表（冪等）
- 新增欄位用 `ALTER TABLE ADD COLUMN`，外面不加 `IF NOT EXISTS`（SQLite 不支援），直接 `let _ = query().execute().await`（忽略錯誤）

### SQLite URL（Windows 路徑）
```rust
let url = format!("sqlite:{}?mode=rwc",
    db_path.to_string_lossy().replace('\\', "/"));
```
必須將反斜線轉正斜線，否則 sqlx 在 Windows 無法開啟。

---

## Tauri Command 慣例

- **純計算**（engine）：`pub fn`，參數直接傳值，不需要 `state`
- **DB 操作**：`pub async fn`，第一個參數 `state: tauri::State<'_, AppState>`
- 錯誤回傳 `Result<T, String>`，用 `.map_err(|e| e.to_string())`
- 所有 command 在 `commands.rs` 定義，在 `lib.rs` 的 `invoke_handler![]` 中註冊

### ⚠️ WebView2 zoom 下的滑鼠座標補正（BF-013）
Tauri `window.set_zoom()` 在 WebView2 上，**mouse/drag event 的 `clientX/Y` 是物理視窗像素（未縮放）**，但 `getBoundingClientRect()` 回傳 CSS 像素（已縮放）。兩者不在同一座標系，不能直接相減。

正確做法：`clientX/Y` 先除以 zoom 再套 CSS 座標計算：
```typescript
const zoom = parseFloat(getComputedStyle(document.documentElement).getPropertyValue('--app-zoom')) || 1;
const yInContent = (clientY / zoom - rect.top) + scrollTop;
```

**`--app-zoom` CSS 變數管理：**
- `main.css` 寫死初始值 `1.3`（防止首幀閃爍）
- `App.vue onMounted`：從 DB 讀取後立即 `setProperty('--app-zoom', zoom)` 同步
- `SettingsModal.applyZoom()`：每次 zoom 變更時一併更新 CSS 變數

---

### ⚠️ JS invoke 參數命名（BF-010）
Tauri v2 的轉換方向是 **Rust snake_case → JS camelCase**。
JS 端 invoke 時必須傳 **camelCase** key，Tauri 會自動轉回 snake_case 給 Rust：
```typescript
// Rust 定義: est_mins: i64
invoke("cmd", { estMins: 60 })   // ✅ 正確
invoke("cmd", { est_mins: 60 })  // ❌ 錯誤，會報 missing key
```
單字參數（`id`, `url`, `date` 等）不受影響。

---

## Modal 慣例

- Modal 元件掛在 `App.vue` 最底層（z-index 安全）
- `modal` ref 型別：`"emergency" | "import" | "extra" | "settings" | null`
- 背景點擊關閉：`@mousedown.self="$emit('close')"`（用 mousedown 避免拖拉誤觸）
- emit：`close`（必）、`saved` / `confirmed` / `imported`（視需要）

---

## Composable 慣例

- **singleton 共享狀態**：`ref` / `reactive` 宣告在 `export function useXxx()` **外部**（module level），讓多個呼叫共享同一份狀態。例：`useLogger`、`useSync`
- **工廠模式**：每次呼叫建立新實例，例：`useTasksDb()`、`useRoomsDb()`
- Tauri invoke 錯誤**必須** `console.error(...)` 才會進 DebugPanel（`invoke()` 不走 `window.fetch`）

---

## 樣式慣例

### Tailwind CSS v4 注意事項
- `@layer components` 內的 `@apply` **不能引用**同一層的其他自訂 class（如 `.form-select { @apply form-input; }` 會在 production build 報錯）
- 自訂 class 必須展開為原生 utility，不能相互 `@apply`

### 全域元件 class（定義於 `src/assets/main.css`）
| Class | 用途 |
|-------|------|
| `.form-label` | `block text-xs font-medium text-gray-400 mb-1` |
| `.form-input` | 深色輸入框，帶 focus ring |
| `.form-select` | 同 form-input 展開版 + appearance-none |
| `.btn-primary` | 藍色主按鈕，disabled 半透明 |
| `.btn-secondary` | 灰色次要按鈕 |
| `.btn-danger` | 紅色危險操作按鈕 |

### 配色系統
- 背景基底：`bg-gray-900`（全頁）、`bg-gray-800`（側欄/卡片）、`bg-gray-700`（輸入框）
- 緊急程度：Trauma=red、Level1=orange、Level2=yellow、Level3=blue、Normal=gray
- 人員類別：SA=blue、OR護理師=green、實習生=yellow、其他單位=purple

---

## Debug Panel 慣例

- `Ctrl+Shift+D` 開啟 / 關閉；`Esc` 關閉
- 無 detail：單擊複製；有 detail：單擊展開、雙擊複製
- `initLogger()` 必須在 `createApp()` **之前**呼叫（`main.ts`）
- 需要 capability：`fs:allow-write-text-file`、`fs:allow-create`（匯出至桌面）

---

## Excel 解析慣例

### 樞紐表格式（房間月排班）
醫院 XLSX 為「週次樞紐表」，解析方式：
1. `XLSX.read(data, { type: "array", raw: true })` — `raw: true` 讓日期序列號以數字回傳
2. `XLSX.utils.sheet_to_json(ws, { header: 1, defval: "", raw: true })` — 以陣列逐列讀取
3. 偵測 `row[0]` 以「第」開頭且以「週」結尾 → week header，`row[2..9]` 為 Mon~Sun 序列日期
4. 偵測 `row[1].toUpperCase() === "AM" | "PM"` 或 `row[1] === "值班" | "值班時段"` → 資料列，`row[2..9]` 為科別（空字串略過）
5. 刀房名稱由 `row[0]` 非空時更新追蹤

### 時段定義（PERIOD_TIMES，秒數自午夜起算）
| 時段識別 | 開始 | 結束 | 說明 |
|----------|------|------|------|
| `AM` | 07:30（27000s） | 12:30（45000s） | 上午刀房 |
| `PM` | 12:30（45000s） | 15:50（57000s） | 下午刀房 |
| `值班` | 15:50（57000s） | 隔日07:30（111600s） | 值班時段，end 用 31\*3600+30\*60 表示跨日 |

Excel 中 col B 可能出現 `"值班"` 或 `"值班時段"` 兩種寫法，一律 map 至 `"值班"` key。

### Excel 序列日期轉換
```typescript
function excelSerialToDateStr(serial: number): string {
  // Excel serial = days since 1899-12-30（含 Lotus 1900 閏年 bug）
  const date = new Date(Math.round((serial - 25569) * 86400 * 1000));
  const y = date.getUTCFullYear();
  const m = String(date.getUTCMonth() + 1).padStart(2, "0");
  const d = String(date.getUTCDate()).padStart(2, "0");
  return `${y}-${m}-${d}`;
}
```
**注意**：必須用 `getUTC*()` 而非 `get*()`，避免時區偏移導致日期差一天。

### 平面格式（人員班表）
`XLSX.read(data, { type: "array", cellDates: true })` + `sheet_to_json(ws, { raw: false, defval: "" })`
輸出為 key-value 物件陣列，欄名以 `STAFF_COL_ALIASES` mapping 正規化為統一 key。

---

## 平台注意事項（Windows）

- SQLite 路徑需將 `\` 轉為 `/`（見 DB 操作慣例）
- Tauri `app_data_dir()` 在 Windows 回傳 `C:\Users\{user}\AppData\Roaming\{identifier}\`
- PowerShell 不支援 `&&` pipeline，用 `;` 或 `if ($?) { ... }` 串聯指令
- `cargo check` 在 Windows 首次執行較慢（LLVM 編譯），後續有 cache 會快很多
