## 🚀 專案初始化（Bootstrap）

**規則：每次對話開始時，檢查以下文件是否存在。若不存在，立即用對應的空白模板建立，再繼續回應使用者。**

| 文件 | 若不存在 |
|------|---------|
| `project.md` | 建立，包含：專案概述、技術架構、目錄結構、資料庫 Schema、已完成功能、已知問題、開發階段歷程 等章節標題（內容留空） |
| `project_conventions.md` | 建立，包含：技術棧快速索引、檔案結構慣例、資料庫操作慣例、Toast 慣例、Composable 慣例、樣式慣例、平台注意事項 等章節標題（內容留空） |
| `project_decisions.md` | 建立，包含說明文字與 ADR 範本格式（背景／決策／理由／否決方案），內容留空 |
| `project_api.md` | 建立，包含：外部 API、內部介面、資料格式備忘 等章節標題（內容留空） |
| `project_bugfix.md` | 建立，包含說明文字與 BF 範本格式（問題描述／嘗試過程／根本原因／最終解法／牽扯檔案），內容留空 |

---

## 作業流程

### 0. 架構先行（Design First）
規則：討論新功能或新模組時，先釐清整體架構與實作細節，確認無誤後再動手寫程式碼。

### 1. 讀取與分析（Context & Analyze）
規則：開始任何修改前，先讀取 `project.md` 掌握現況。需修改現有程式碼時，先閱讀相關檔案，分析現有邏輯後再執行，不憑假設動手。

**專案參考文件索引：**
| 文件 | 用途 |
|------|------|
| `project.md` | 架構總覽、已完成功能、已知問題 |
| `project_conventions.md` | 程式碼慣例、樣式層次、平台注意事項 |
| `project_decisions.md` | 技術決策紀錄，避免重複討論已決定的方案 |
| `project_api.md` | API 合約、內部介面、資料格式備忘 |
| `project_bugfix.md` | 踩過的坑與解法，避免重蹈覆轍 |

### 2. 最小範圍修改（Minimal & Safe Coding）
規則：
- 只修改被要求的部分，不主動重構、不新增沒被要求的功能、不添加多餘的註解或 TODO。
- 使用工具（Read → Edit/Write）直接操作檔案，修改前必須先讀取目標檔案。
- 修改範圍盡可能小而精準，避免大範圍覆蓋造成難以追蹤的副作用。

### 3. 驗證與除錯紀錄（Verify & Bugfix）
規則：確保程式碼邏輯正確後再進行下一步。若遇重要技術 Bug 且修改三次（含）以上才解決，或問題根因具有平台特殊性，將問題描述、牽扯的檔案與函式名稱、以及最終有效的解決方式，精煉記錄於 `project_bugfix.md`，作為未來的避坑指南。

### 4. 總結與更新（Update Blueprint）
規則：修改與驗證全數完成後，更新 `project.md`，將本次變更（新增功能、修復項目、架構調整等）精簡扼要地反映到對應章節，維持專案藍圖的最新狀態。

---

## 🛠️ 通用元件模式

### Debug 面板（DebugPanel）

適用於 Tauri + Vue3 專案，提供開發期與上線後的即時 log 查閱與匯出能力。

#### 架構
| 檔案 | 職責 |
|------|------|
| `src/composables/useLogger.ts` | singleton logger，攔截全域錯誤與 fetch，暴露 `logs / addLog / initLogger / exportLogs / clearLogs` |
| `src/components/DebugPanel.vue` | 浮動面板 UI，固定右下角，顯示 log 條目、展開 stack trace、複製、匯出 |
| `src/main.ts` | 在 `createApp()` 前呼叫 `useLogger().initLogger()` 以啟動攔截 |
| `src/App.vue` | 宣告 `debugOpen ref`，監聽 `Ctrl+Shift+D` 切換，掛載 `<DebugPanel v-if="debugOpen" />` |

#### useLogger 行為
- 最多保留 300 筆（FIFO）
- 攔截：`window.onerror`、`unhandledrejection`、`console.error`、`console.warn`、全域 `fetch`（記錄 status ≥ 400 或 catch）
- `exportLogs()` 使用 `@tauri-apps/plugin-fs` 寫出至桌面（`$DESKTOP/medbase-log-{timestamp}.txt`）
- log 物件：`{ id, time, level: 'error'|'warn'|'info', message, detail? }`

#### DebugPanel 互動
- 無 detail 的條目：**單擊**複製，訊息短暫變綠顯示「✓ 已複製」
- 有 detail 的條目：**單擊**展開/收合；**雙擊**複製；展開後點 detail 區塊也複製完整記錄
- 複製格式：`[時間] LEVEL 訊息\nstack trace`
- Header 按鈕：匯出（寫桌面）、清除

#### 快速鍵
| 快速鍵 | 動作 |
|--------|------|
| `Ctrl+Shift+D` | 開啟 / 關閉 Debug 面板 |
| `Esc` | 關閉面板 |

#### 注意事項
- `initLogger()` 必須在 Vue app 掛載前執行，否則 Vue 內部 warn 無法被攔截
- `exportLogs` 需要 `fs:allow-write-text-file` 與 `fs:allow-create` capability（`$HOME/**`）
- Panel 使用 `z-[9999]` 確保浮在所有 overlay 之上