# 專案慣例 (Project Conventions)

## 技術棧快速索引

| 層次 | 技術 | 版本 |
|------|------|------|
| 前端框架 | Vue 3 (Composition API) | latest |
| 建構工具 | Vite | latest |
| 樣式 | Tailwind CSS | latest |
| 圖示 | FontAwesome | latest |
| 桌面框架 | Tauri 2.0 | 2.x |
| 後端語言 | Rust | stable |
| 本地資料庫 | SQLite (sqlx) | latest |

## 檔案結構慣例

```
src/
  components/     # Vue 元件
  composables/    # Vue Composables (useXxx 命名)
  views/          # 頁面級元件
  stores/         # Pinia stores
  types/          # TypeScript 型別定義
  utils/          # 工具函數
src-tauri/
  src/
    commands/     # Tauri command handlers
    db/           # SQLite 資料庫邏輯
    models/       # Rust 資料模型
```

## 資料庫操作慣例

## Toast 慣例

## Composable 慣例

- 檔名以 `use` 開頭（如 `useLogger.ts`）
- 回傳 reactive 狀態與操作函式

## 樣式慣例

- 使用 Tailwind CSS utility classes
- 自訂樣式放於 `src/assets/`
- 元件樣式使用 `<style scoped>`

## 平台注意事項
