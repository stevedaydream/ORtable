# 專案文件建立模板

建立以下五份文件時使用此模板。每份文件建立後立即填入可從程式碼推導的基本資訊。

---

## project.md

```markdown
# 專案名稱

## 1. 專案目標與背景

## 2. 系統架構（技術棧）

## 3. UI/UX 設計規範

## 4. 核心業務邏輯

## 5. 核心資料模型

## 6. 開發階段規劃（Phases）

## 7. 目錄結構

## 8. 已知問題 / 待辦
```

---

## project_conventions.md

```markdown
# 專案慣例

## 技術棧快速索引

## 檔案結構慣例

## 資料庫操作慣例

## Tauri Command 慣例

## Modal 慣例

## Composable 慣例

## 樣式慣例

## 平台注意事項
```

---

## project_decisions.md

```markdown
# 技術決策紀錄（Architecture Decision Records）

記錄重要技術決策，避免未來重複討論已決定的方案。

## ADR 格式

### ADR-XXX: 標題

**背景：** 為什麼需要做這個決策？

**決策：** 最終選擇什麼？

**理由：** 為什麼選這個？

**否決方案：** 哪些方案被否決，為什麼？
```

---

## project_api.md

```markdown
# API 與介面備忘

## 外部 API

## Tauri Commands（內部介面）

## 資料格式備忘
```

---

## project_bugfix.md

```markdown
# 除錯紀錄（Bug Fix Log）

記錄修改三次（含）以上才解決的重要 Bug，以及具有平台特殊性的問題，作為未來避坑指南。

## BF 格式

### BF-XXX: 標題

**問題描述：**

**嘗試過程：**

**根本原因：**

**最終解法：**

**牽扯檔案：**
```
