use sqlx::{Row, SqlitePool};
use crate::models::RoomScheduleEntry;

fn map_row(r: &sqlx::sqlite::SqliteRow) -> RoomScheduleEntry {
    RoomScheduleEntry {
        id:           r.get("id"),
        room_name:    r.get("room_name"),
        dept:         r.get("dept"),
        date:         r.get("date"),
        start_time:   r.get("start_time"),
        end_time:     r.get("end_time"),
        notes:        r.get("notes"),
        is_emergency: r.get::<i64, _>("is_emergency") != 0,
    }
}

pub async fn get_by_date(pool: &SqlitePool, date: &str) -> Result<Vec<RoomScheduleEntry>, String> {
    sqlx::query(
        "SELECT * FROM room_shifts WHERE date=? ORDER BY room_name ASC, start_time ASC"
    )
    .bind(date).fetch_all(pool).await
    .map(|rows| rows.iter().map(map_row).collect())
    .map_err(|e| e.to_string())
}

pub async fn replace_by_month(
    pool: &SqlitePool,
    _unused_month: &str, // Keep signature for now but ignore
    entries: &[RoomScheduleEntry],
) -> Result<(), String> {
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    // 1. 找出資料中所有出現過的月份 (YYYY-MM)
    let mut months = std::collections::HashSet::new();
    for e in entries {
        if e.date.len() >= 7 {
            months.insert(&e.date[0..7]);
        }
    }

    // 2. 刪除這些月份的所有既有時段，避免重複
    for month in months {
        sqlx::query("DELETE FROM room_shifts WHERE date LIKE ?")
            .bind(format!("{}%", month))
            .execute(&mut *tx).await.map_err(|e| e.to_string())?;
    }

    // 3. 批次寫入新資料
    for e in entries {
        sqlx::query(
            "INSERT INTO room_shifts (room_name,dept,date,start_time,end_time,notes,is_emergency)
             VALUES (?,?,?,?,?,?,?)",
        )
        .bind(&e.room_name).bind(&e.dept).bind(&e.date)
        .bind(e.start_time).bind(e.end_time).bind(&e.notes)
        .bind(e.is_emergency as i64)
        .execute(&mut *tx).await.map_err(|e| e.to_string())?;
    }
    tx.commit().await.map_err(|e| e.to_string())
}

pub async fn replace_by_date(
    pool: &SqlitePool,
    date: &str,
    entries: &[RoomScheduleEntry],
) -> Result<(), String> {
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM room_shifts WHERE date=?")
        .bind(date).execute(&mut *tx).await.map_err(|e| e.to_string())?;
    for e in entries {
        sqlx::query(
            "INSERT INTO room_shifts (room_name,dept,date,start_time,end_time,notes,is_emergency) VALUES (?,?,?,?,?,?,?)"
        )
        .bind(&e.room_name).bind(&e.dept).bind(date)
        .bind(e.start_time).bind(e.end_time).bind(&e.notes)
        .bind(e.is_emergency as i64)
        .execute(&mut *tx).await.map_err(|e| e.to_string())?;
    }
    tx.commit().await.map_err(|e| e.to_string())
}
