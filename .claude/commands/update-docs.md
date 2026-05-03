# 更新專案文件

依序執行以下步驟，將本次對話的變更精簡反映至 project.md 系列檔案。

---

## 步驟 1：掌握本次變更範圍

執行以下指令取得本次修改的檔案清單與 diff 摘要：

```bash
git diff --stat HEAD
git diff HEAD --unified=3
```

若有未追蹤的新檔案一併納入考量（`git status`）。

---

## 步驟 2：逐一判斷哪些文件需要更新

| 文件 | 何時需要更新 |
|------|------------|
| `project.md` | 新增功能、新 Phase、架構異動、已知問題變動 |
| `project_api.md` | 新增或修改 Tauri Command、新增 DB 欄位或資料表、TS 介面變動 |
| `project_conventions.md` | 發現新的慣例或修改了既有慣例 |
| `project_decisions.md` | 做了技術選型決策（不是 bugfix，是「我們決定用 X 而非 Y」） |
| `project_bugfix.md` | 修了 3 次以上才解決的 Bug、或具有平台特殊性的問題 |

**不需要更新的情況：** 小幅 UI 調整、純粹的 label 文字修改、單純修正 typo。

---

## 步驟 3：更新各文件

### project.md 更新原則
- 在「開發階段規劃」補上本次 Phase（若功能夠大）或在現有 Phase 內補條目
- 在「已知問題 / 待辦」移除已解決項目、新增新發現的問題
- **只改動有變化的章節，其餘保持原樣**

### project_api.md 更新原則
- 每個新 Tauri Command 一行：`command_name(param: type) → ReturnType` 加簡短說明
- 新增的 DB 欄位標記在對應資料表下方
- **JS↔Rust 參數命名陷阱若已記錄則跳過**

### project_bugfix.md 更新原則
- 使用 BF-XXX 格式（接續現有最大編號）
- 必填：問題描述、根本原因、最終解法、牽扯檔案
- 只記錄「反覆發生」或「原因非顯而易見」的問題

### project_conventions.md 更新原則
- 若本次引入了新的程式碼模式（例如新的 emit 使用方式），才更新
- 不記錄只用一次的特例

### project_decisions.md 更新原則
- ADR 只記錄「有明確被否決的替代方案」的決策
- 純粹 bugfix 不算 ADR

---

## 步驟 4：確認完成

更新後簡短回報：
- 更動了哪些文件、哪些章節
- 哪些文件判斷不需要更新（及原因）
