# DebugPanel 架構參考

適用於本專案（Tauri + Vue3），提供開發期與上線後的即時 log 查閱與匯出能力。

## 檔案職責

| 檔案 | 職責 |
|------|------|
| `src/composables/useLogger.ts` | singleton logger，攔截全域錯誤與 fetch，暴露 `logs / addLog / initLogger / exportLogs / clearLogs` |
| `src/components/DebugPanel.vue` | 浮動面板 UI，固定右下角，顯示 log 條目、展開 stack trace、複製、匯出 |
| `src/main.ts` | 在 `createApp()` 前呼叫 `useLogger().initLogger()` 啟動攔截 |
| `src/App.vue` | `debugOpen ref` + `Ctrl+Shift+D` 監聽 + `<DebugPanel v-if="debugOpen" />` |

## useLogger 行為

- 最多保留 300 筆（FIFO）
- 攔截：`window.onerror`、`unhandledrejection`、`console.error`、`console.warn`、全域 `fetch`（status ≥ 400 或 catch）
- `exportLogs()` 使用 `@tauri-apps/plugin-fs` 寫至桌面（`$DESKTOP/medbase-log-{timestamp}.txt`）
- log 物件：`{ id, time, level: 'error'|'warn'|'info', message, detail? }`

## Panel 互動行為

- 無 detail：**單擊**複製，短暫顯示「✓ 已複製」
- 有 detail：**單擊**展開/收合；**雙擊**複製；展開後點 detail 區塊也複製完整記錄
- 複製格式：`[時間] LEVEL 訊息\nstack trace`
- Header 按鈕：匯出（寫桌面）、清除

## 快速鍵

| 快速鍵 | 動作 |
|--------|------|
| `Ctrl+Shift+D` | 開啟 / 關閉 |
| `Esc` | 關閉 |

## 重要注意事項

- `initLogger()` 必須在 `createApp()` **之前**呼叫，否則 Vue 內部 warn 無法被攔截
- `exportLogs` 需要 capability：`fs:allow-write-text-file`、`fs:allow-create`（`$HOME/**`）
- Panel 使用 `z-[9999]`，確保浮在所有 overlay 之上
- Tauri `invoke()` 走 IPC channel，**不經過** `window.fetch`，GAS / Rust 端錯誤需在 catch block 補 `console.error()` 才會進 DebugPanel（見 BF-004）
