# 新功能實作檢查清單

## Phase 0：設計確認（動手前必做）

- [ ] 釐清資料模型：新增哪些欄位 / 表？影響哪些現有模型？
- [ ] 釐清前後端介面：需要幾支 Tauri command？參數與回傳型別？
- [ ] 釐清 UI 流程：從哪裡觸發？哪個元件負責？要不要新增 Modal / Panel？
- [ ] 確認不影響現有功能（特別是 store 的 load / edit / remove 流程）

## Phase 1：後端（Rust）

- [ ] `models.rs`：新增 / 擴充 struct
- [ ] `db/mod.rs`：migrate 新增 CREATE TABLE 或 ALTER TABLE
- [ ] `db/xxx.rs`：實作 CRUD 函數（用 `sqlx::query()` runtime API，不用 macro）
- [ ] `db/mod.rs`：宣告新模組 `pub mod xxx;`
- [ ] `commands.rs`：新增 command handler
- [ ] `lib.rs`：在 `invoke_handler![]` 註冊新 command
- [ ] 執行 `cargo check` 確認無編譯錯誤

## Phase 2：前端（Vue / TypeScript）

- [ ] `types/index.ts`：新增對應 TypeScript interface
- [ ] `composables/useDatabase.ts`：新增 invoke wrapper 函數
- [ ] 若需要全域狀態：新增或擴充 `stores/xxx.ts`（Pinia setup store）
- [ ] 實作 UI 元件（Modal / Panel / 卡片）
- [ ] 在 `App.vue` 掛載新 Modal（若需要）並擴充 `modal` ref 型別
- [ ] 執行 `npx vue-tsc --noEmit` 確認無型別錯誤

## Phase 3：驗證

- [ ] 功能主流程正常運作
- [ ] 邊界情況：空資料、重複操作、錯誤回傳
- [ ] 重新整理（F5 / reload）後資料正確從 DB 恢復
- [ ] 不影響其他現有功能

## Phase 4：收尾

- [ ] 更新 `project.md`（新 Phase 或已完成功能描述）
- [ ] 若遇到平台特殊 Bug 修了 3 次以上：記錄至 `project_bugfix.md`
- [ ] 若有重要技術決策：記錄至 `project_decisions.md`
