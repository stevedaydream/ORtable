use sqlx::{Row, SqlitePool};
use crate::models::Room;

fn map_row(r: &sqlx::sqlite::SqliteRow) -> Room {
    Room {
        id:            r.get("id"),
        name:          r.get("name"),
        display_order: r.get("display_order"),
        is_backup:     r.get::<i64, _>("is_backup") != 0,
    }
}

pub async fn get_all(pool: &SqlitePool) -> Result<Vec<Room>, String> {
    sqlx::query("SELECT * FROM rooms ORDER BY display_order ASC, id ASC")
        .fetch_all(pool).await
        .map(|rows| rows.iter().map(map_row).collect())
        .map_err(|e| e.to_string())
}

pub async fn create(pool: &SqlitePool, r: &Room) -> Result<Room, String> {
    let id = sqlx::query(
        "INSERT INTO rooms (name, display_order, is_backup) VALUES (?,?,?)"
    )
    .bind(&r.name).bind(r.display_order).bind(r.is_backup as i64)
    .execute(pool).await.map_err(|e| e.to_string())?
    .last_insert_rowid();
    fetch_one(pool, id).await
}

pub async fn update(pool: &SqlitePool, r: &Room) -> Result<Room, String> {
    sqlx::query(
        "UPDATE rooms SET name=?, display_order=?, is_backup=? WHERE id=?"
    )
    .bind(&r.name).bind(r.display_order).bind(r.is_backup as i64).bind(r.id)
    .execute(pool).await.map_err(|e| e.to_string())?;
    fetch_one(pool, r.id).await
}

pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), String> {
    sqlx::query("DELETE FROM rooms WHERE id=?")
        .bind(id).execute(pool).await.map_err(|e| e.to_string())?;
    Ok(())
}

async fn fetch_one(pool: &SqlitePool, id: i64) -> Result<Room, String> {
    sqlx::query("SELECT * FROM rooms WHERE id=?")
        .bind(id).fetch_one(pool).await
        .map(|r| map_row(&r))
        .map_err(|e| e.to_string())
}
