use sqlx::{Row, SqlitePool};
use crate::models::Staff;

fn map_row(r: &sqlx::sqlite::SqliteRow) -> Staff {
    Staff {
        id:                   r.get("id"),
        name:                 r.get("name"),
        role:                 r.get("role"),
        staff_type:           r.get("type"),
        staff_category:       r.get("staff_category"),
        unit:                 r.get("unit"),
        is_on_call:           r.get::<i64, _>("is_on_call") != 0,
        is_volunteer_extra:   r.get::<i64, _>("is_volunteer_extra") != 0,
        today_shift_start:    r.get("today_shift_start"),
        next_day_shift_start: r.get("next_day_shift_start"),
    }
}

pub async fn get_all(pool: &SqlitePool) -> Result<Vec<Staff>, String> {
    sqlx::query("SELECT * FROM staff ORDER BY id ASC")
        .fetch_all(pool).await
        .map(|rows| rows.iter().map(map_row).collect())
        .map_err(|e| e.to_string())
}

pub async fn create(pool: &SqlitePool, s: &Staff) -> Result<Staff, String> {
    let id = sqlx::query(
        "INSERT INTO staff (name,role,type,staff_category,unit,is_on_call,is_volunteer_extra,
         today_shift_start,next_day_shift_start) VALUES (?,?,?,?,?,?,?,?,?)",
    )
    .bind(&s.name).bind(&s.role).bind(&s.staff_type)
    .bind(&s.staff_category).bind(&s.unit)
    .bind(s.is_on_call as i64).bind(s.is_volunteer_extra as i64)
    .bind(s.today_shift_start).bind(s.next_day_shift_start)
    .execute(pool).await.map_err(|e| e.to_string())?
    .last_insert_rowid();

    fetch_one(pool, id).await
}

pub async fn update(pool: &SqlitePool, s: &Staff) -> Result<Staff, String> {
    sqlx::query(
        "UPDATE staff SET name=?,role=?,type=?,staff_category=?,unit=?,
         is_on_call=?,is_volunteer_extra=?,today_shift_start=?,next_day_shift_start=?
         WHERE id=?",
    )
    .bind(&s.name).bind(&s.role).bind(&s.staff_type)
    .bind(&s.staff_category).bind(&s.unit)
    .bind(s.is_on_call as i64).bind(s.is_volunteer_extra as i64)
    .bind(s.today_shift_start).bind(s.next_day_shift_start).bind(s.id)
    .execute(pool).await.map_err(|e| e.to_string())?;

    fetch_one(pool, s.id).await
}

pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), String> {
    sqlx::query("DELETE FROM staff WHERE id=?")
        .bind(id).execute(pool).await.map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn replace_all(pool: &SqlitePool, list: &[Staff]) -> Result<(), String> {
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM staff")
        .execute(&mut *tx).await.map_err(|e| e.to_string())?;
    for s in list {
        sqlx::query(
            "INSERT INTO staff (id,name,role,type,staff_category,unit,is_on_call,
             is_volunteer_extra,today_shift_start,next_day_shift_start)
             VALUES (?,?,?,?,?,?,?,?,?,?)",
        )
        .bind(s.id).bind(&s.name).bind(&s.role).bind(&s.staff_type)
        .bind(&s.staff_category).bind(&s.unit)
        .bind(s.is_on_call as i64).bind(s.is_volunteer_extra as i64)
        .bind(s.today_shift_start).bind(s.next_day_shift_start)
        .execute(&mut *tx).await.map_err(|e| e.to_string())?;
    }
    tx.commit().await.map_err(|e| e.to_string())
}

async fn fetch_one(pool: &SqlitePool, id: i64) -> Result<Staff, String> {
    sqlx::query("SELECT * FROM staff WHERE id=?")
        .bind(id).fetch_one(pool).await
        .map(|r| map_row(&r))
        .map_err(|e| e.to_string())
}
