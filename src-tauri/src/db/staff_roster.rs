use sqlx::{Row, SqlitePool};
use crate::models::StaffRosterEntry;

fn map_row(r: &sqlx::sqlite::SqliteRow) -> StaffRosterEntry {
    StaffRosterEntry {
        id:         r.get("id"),
        staff_name: r.get("staff_name"),
        date:       r.get("date"),
        shift_name: r.get("shift_name"),
    }
}

pub async fn get_by_date(pool: &SqlitePool, date: &str) -> Result<Vec<StaffRosterEntry>, String> {
    sqlx::query("SELECT * FROM staff_roster WHERE date=?")
        .bind(date).fetch_all(pool).await
        .map(|rows| rows.iter().map(map_row).collect())
        .map_err(|e| e.to_string())
}

pub async fn replace_by_month(
    pool: &SqlitePool,
    _unused_month: &str,
    entries: &[StaffRosterEntry],
) -> Result<(), String> {
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    // 1. 找出資料中所有出現過的月份
    let mut months = std::collections::HashSet::new();
    for e in entries {
        if e.date.len() >= 7 {
            months.insert(&e.date[0..7]);
        }
    }

    // 2. 刪除對應月份
    for month in months {
        sqlx::query("DELETE FROM staff_roster WHERE date LIKE ?")
            .bind(format!("{}%", month))
            .execute(&mut *tx).await.map_err(|e| e.to_string())?;
    }

    // 3. 批次寫入
    for e in entries {
        sqlx::query("INSERT INTO staff_roster (staff_name, date, shift_name) VALUES (?,?,?)")
            .bind(&e.staff_name).bind(&e.date).bind(&e.shift_name)
            .execute(&mut *tx).await.map_err(|e| e.to_string())?;
    }
    tx.commit().await.map_err(|e| e.to_string())
}
