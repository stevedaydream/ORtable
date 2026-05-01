use sqlx::{Row, SqlitePool};

pub async fn get(pool: &SqlitePool, key: &str) -> Result<Option<String>, String> {
    sqlx::query("SELECT value FROM settings WHERE key=?")
        .bind(key)
        .fetch_optional(pool)
        .await
        .map(|opt| opt.map(|r| r.get("value")))
        .map_err(|e| e.to_string())
}

pub async fn set(pool: &SqlitePool, key: &str, value: &str) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO settings (key, value) VALUES (?, ?)
         ON CONFLICT(key) DO UPDATE SET value=excluded.value",
    )
    .bind(key)
    .bind(value)
    .execute(pool)
    .await
    .map(|_| ())
    .map_err(|e| e.to_string())
}
