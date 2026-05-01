use sqlx::{Row, SqlitePool};
use crate::models::RoomScheduleEntry;

fn map_row(r: &sqlx::sqlite::SqliteRow) -> RoomScheduleEntry {
    RoomScheduleEntry {
        id:         r.get("id"),
        room_name:  r.get("room_name"),
        dept:       r.get("dept"),
        date:       r.get("date"),
        start_time: r.get("start_time"),
        end_time:   r.get("end_time"),
        notes:      r.get("notes"),
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
    month: &str,
    entries: &[RoomScheduleEntry],
) -> Result<(), String> {
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM room_shifts WHERE date LIKE ?")
        .bind(format!("{}%", month))
        .execute(&mut *tx).await.map_err(|e| e.to_string())?;
    for e in entries {
        sqlx::query(
            "INSERT INTO room_shifts (room_name,dept,date,start_time,end_time,notes)
             VALUES (?,?,?,?,?,?)",
        )
        .bind(&e.room_name).bind(&e.dept).bind(&e.date)
        .bind(e.start_time).bind(e.end_time).bind(&e.notes)
        .execute(&mut *tx).await.map_err(|e| e.to_string())?;
    }
    tx.commit().await.map_err(|e| e.to_string())
}
