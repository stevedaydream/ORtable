use sqlx::{Row, SqlitePool};
use crate::models::StaffAssignment;

fn map_row(r: &sqlx::sqlite::SqliteRow) -> StaffAssignment {
    StaffAssignment {
        id:         r.get("id"),
        staff_name: r.get("staff_name"),
        room_name:  r.get("room_name"),
        date:       r.get("date"),
        role:       r.get("role"),
    }
}

pub async fn get_by_date(pool: &SqlitePool, date: &str) -> Result<Vec<StaffAssignment>, String> {
    sqlx::query(
        "SELECT * FROM staff_assignments WHERE date=? ORDER BY room_name ASC, role ASC"
    )
    .bind(date).fetch_all(pool).await
    .map(|rows| rows.iter().map(map_row).collect())
    .map_err(|e| e.to_string())
}

pub async fn add_one(pool: &SqlitePool, entry: &StaffAssignment) -> Result<StaffAssignment, String> {
    let id = sqlx::query(
        "INSERT INTO staff_assignments (staff_name, room_name, date, role) VALUES (?,?,?,?)"
    )
    .bind(&entry.staff_name).bind(&entry.room_name).bind(&entry.date).bind(&entry.role)
    .execute(pool).await.map_err(|e| e.to_string())?.last_insert_rowid();
    Ok(StaffAssignment { id, ..entry.clone() })
}

pub async fn remove_one(pool: &SqlitePool, id: i64) -> Result<(), String> {
    sqlx::query("DELETE FROM staff_assignments WHERE id=?")
        .bind(id).execute(pool).await.map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn replace_by_month(
    pool: &SqlitePool,
    _unused_month: &str,
    entries: &[StaffAssignment],
) -> Result<(), String> {
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    let mut months = std::collections::HashSet::new();
    for e in entries {
        if e.date.len() >= 7 {
            months.insert(&e.date[0..7]);
        }
    }

    for month in months {
        sqlx::query("DELETE FROM staff_assignments WHERE date LIKE ?")
            .bind(format!("{}%", month))
            .execute(&mut *tx).await.map_err(|e| e.to_string())?;
    }

    for e in entries {
        sqlx::query(
            "INSERT INTO staff_assignments (staff_name, room_name, date, role) VALUES (?,?,?,?)",
        )
        .bind(&e.staff_name).bind(&e.room_name).bind(&e.date).bind(&e.role)
        .execute(&mut *tx).await.map_err(|e| e.to_string())?;
    }
    tx.commit().await.map_err(|e| e.to_string())
}
