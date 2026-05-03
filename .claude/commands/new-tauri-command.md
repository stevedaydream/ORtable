# 新增 Tauri Command 完整流程

每次新增一個資料庫相關功能，依序完成以下步驟。

## 1. DB 遷移（`src-tauri/src/db/mod.rs`）

```rust
// migrate() 函數內，CREATE TABLE IF NOT EXISTS（冪等）
sqlx::query("CREATE TABLE IF NOT EXISTS my_table (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL
)").execute(pool).await?;

// 補欄位用 ALTER TABLE，直接忽略錯誤（SQLite 不支援 IF NOT EXISTS）
let _ = sqlx::query("ALTER TABLE my_table ADD COLUMN notes TEXT NOT NULL DEFAULT ''")
  .execute(pool).await;
```

## 2. Rust 資料模型（`src-tauri/src/models.rs`）

```rust
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct MyItem {
    pub id: i64,
    pub name: String,
    pub notes: String,
}
```

## 3. DB 模組（`src-tauri/src/db/my_table.rs`）

```rust
use sqlx::{SqlitePool, Row};
use crate::models::MyItem;

fn map_row(r: &sqlx::sqlite::SqliteRow) -> MyItem {
    MyItem {
        id:    r.get("id"),
        name:  r.get("name"),
        notes: r.get("notes"),
    }
}

pub async fn get_all(pool: &SqlitePool) -> Result<Vec<MyItem>, sqlx::Error> {
    sqlx::query("SELECT * FROM my_table ORDER BY id")
        .fetch_all(pool).await
        .map(|rows| rows.iter().map(map_row).collect())
}

pub async fn create(pool: &SqlitePool, name: &str, notes: &str) -> Result<MyItem, sqlx::Error> {
    let id = sqlx::query("INSERT INTO my_table (name, notes) VALUES (?, ?)")
        .bind(name).bind(notes)
        .execute(pool).await?.last_insert_rowid();
    Ok(MyItem { id, name: name.into(), notes: notes.into() })
}

pub async fn update(pool: &SqlitePool, item: &MyItem) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE my_table SET name=?, notes=? WHERE id=?")
        .bind(&item.name).bind(&item.notes).bind(item.id)
        .execute(pool).await.map(|_| ())
}

pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM my_table WHERE id=?")
        .bind(id).execute(pool).await.map(|_| ())
}
```

## 4. 在 `db/mod.rs` 加入模組宣告

```rust
pub mod my_table;
```

## 5. Command Handler（`src-tauri/src/commands.rs`）

```rust
#[tauri::command]
pub async fn get_all_my_items(state: tauri::State<'_, AppState>) -> Result<Vec<MyItem>, String> {
    db::my_table::get_all(&state.db).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_my_item(state: tauri::State<'_, AppState>, name: String, notes: String) -> Result<MyItem, String> {
    db::my_table::create(&state.db, &name, &notes).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_my_item(state: tauri::State<'_, AppState>, item: MyItem) -> Result<(), String> {
    db::my_table::update(&state.db, &item).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_my_item(state: tauri::State<'_, AppState>, id: i64) -> Result<(), String> {
    db::my_table::delete(&state.db, id).await.map_err(|e| e.to_string())
}
```

## 6. 註冊 Command（`src-tauri/src/lib.rs`）

在 `invoke_handler![]` 陣列內加入：

```rust
commands::get_all_my_items,
commands::create_my_item,
commands::update_my_item,
commands::delete_my_item,
```

## 7. TypeScript 型別（`src/types/index.ts`）

```typescript
export interface MyItem {
  id: number;
  name: string;
  notes: string;
}
```

## 8. useDatabase Wrapper（`src/composables/useDatabase.ts`）

```typescript
export function useMyItemDb() {
  const getAll   = () => invoke<MyItem[]>("get_all_my_items");
  const create   = (name: string, notes: string) => invoke<MyItem>("create_my_item", { name, notes });
  const update   = (item: MyItem) => invoke<void>("update_my_item", { item });
  const remove   = (id: number) => invoke<void>("delete_my_item", { id });
  return { getAll, create, update, remove };
}
```

## 注意事項

- Boolean 欄位用 INTEGER 存（讀：`r.get::<i64,_>("col") != 0`；寫：`val as i64`）
- Windows SQLite 路徑：`path.to_string_lossy().replace('\\', "/")`
- 批次覆寫用 transaction：`let mut tx = pool.begin().await?; ... tx.commit().await?`
- invoke() 錯誤不走 window.fetch，catch 裡必須加 `console.error(...)` 才進 DebugPanel
