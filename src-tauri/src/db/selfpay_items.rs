use sqlx::{Row, SqlitePool};
use crate::models::SelfPayItem;

fn map_row(r: &sqlx::sqlite::SqliteRow) -> SelfPayItem {
    SelfPayItem {
        id:    r.get("id"),
        name:  r.get("name"),
        price: r.get("price"),
        notes: r.get("notes"),
    }
}

pub async fn get_all(pool: &SqlitePool) -> Result<Vec<SelfPayItem>, String> {
    sqlx::query("SELECT id, name, price, notes FROM selfpay_items ORDER BY name ASC")
        .fetch_all(pool)
        .await
        .map(|rows| rows.iter().map(map_row).collect())
        .map_err(|e| e.to_string())
}

pub async fn create(pool: &SqlitePool, item: &SelfPayItem) -> Result<SelfPayItem, String> {
    let id = sqlx::query(
        "INSERT INTO selfpay_items (name, price, notes) VALUES (?, ?, ?)",
    )
    .bind(&item.name)
    .bind(item.price)
    .bind(&item.notes)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?
    .last_insert_rowid();

    Ok(SelfPayItem { id, name: item.name.clone(), price: item.price, notes: item.notes.clone() })
}

pub async fn update(pool: &SqlitePool, item: &SelfPayItem) -> Result<SelfPayItem, String> {
    sqlx::query(
        "UPDATE selfpay_items SET name=?, price=?, notes=? WHERE id=?",
    )
    .bind(&item.name)
    .bind(item.price)
    .bind(&item.notes)
    .bind(item.id)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(item.clone())
}

pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), String> {
    sqlx::query("DELETE FROM selfpay_items WHERE id=?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
