use sqlx::{Row, SqlitePool};
use crate::models::{SurgeryTask, UrgencyLevel};

fn str_to_urgency(s: &str) -> UrgencyLevel {
    match s {
        "Trauma" => UrgencyLevel::Trauma,
        "Level1" => UrgencyLevel::Level1,
        "Level2" => UrgencyLevel::Level2,
        "Level3" => UrgencyLevel::Level3,
        _ => UrgencyLevel::Normal,
    }
}

fn urgency_str(u: &UrgencyLevel) -> &'static str {
    match u {
        UrgencyLevel::Trauma => "Trauma",
        UrgencyLevel::Level1 => "Level1",
        UrgencyLevel::Level2 => "Level2",
        UrgencyLevel::Level3 => "Level3",
        UrgencyLevel::Normal => "Normal",
    }
}

fn map_row(r: &sqlx::sqlite::SqliteRow) -> SurgeryTask {
    let urgency_str: String = r.get("urgency");
    SurgeryTask {
        id:            r.get("id"),
        seq_no:        r.get("seq_no"),
        patient_name:  r.get("patient_name"),
        gender:        r.get("gender"),
        age:           r.get("age"),
        chart_no:      r.get("chart_no"),
        bed_no:        r.get("bed_no"),
        dept:          r.get("dept"),
        diagnosis:     r.get("diagnosis"),
        body_part:     r.get("body_part"),
        procedure:     r.get("procedure"),
        anesthesia:    r.get("anesthesia"),
        surgeon:       r.get("surgeon"),
        vs_note:       r.get("vs_note"),
        expected_room: r.get("expected_room"),
        urgency:       str_to_urgency(&urgency_str),
        scheduled_at:  r.get("scheduled_at"),
        created_at:    r.get("created_at"),
        est_time_mins: r.get::<i64, _>("est_time_mins") as i32,
        status:        r.get("status"),
    }
}

pub async fn get_all(pool: &SqlitePool) -> Result<Vec<SurgeryTask>, String> {
    sqlx::query("SELECT * FROM surgery_tasks ORDER BY created_at ASC")
        .fetch_all(pool)
        .await
        .map(|rows| rows.iter().map(map_row).collect())
        .map_err(|e| e.to_string())
}

pub async fn create(pool: &SqlitePool, task: &SurgeryTask) -> Result<SurgeryTask, String> {
    let now = now_secs();
    let id = sqlx::query(
        "INSERT INTO surgery_tasks
         (seq_no,patient_name,gender,age,chart_no,bed_no,dept,diagnosis,body_part,procedure,anesthesia,surgeon,vs_note,expected_room,urgency,scheduled_at,created_at,est_time_mins,status)
         VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
    )
    .bind(task.seq_no).bind(&task.patient_name).bind(&task.gender).bind(task.age)
    .bind(&task.chart_no).bind(&task.bed_no).bind(&task.dept).bind(&task.diagnosis)
    .bind(&task.body_part).bind(&task.procedure).bind(&task.anesthesia).bind(&task.surgeon)
    .bind(&task.vs_note).bind(&task.expected_room).bind(urgency_str(&task.urgency))
    .bind(task.scheduled_at).bind(now)
    .bind(task.est_time_mins as i64).bind(&task.status)
    .execute(pool).await.map_err(|e| e.to_string())?
    .last_insert_rowid();

    fetch_one(pool, id).await
}

pub async fn update(pool: &SqlitePool, task: &SurgeryTask) -> Result<SurgeryTask, String> {
    sqlx::query(
        "UPDATE surgery_tasks SET
         seq_no=?,patient_name=?,gender=?,age=?,chart_no=?,bed_no=?,dept=?,diagnosis=?,body_part=?,
         procedure=?,anesthesia=?,surgeon=?,vs_note=?,expected_room=?,urgency=?,
         scheduled_at=?,est_time_mins=?,status=?
         WHERE id=?",
    )
    .bind(task.seq_no).bind(&task.patient_name).bind(&task.gender).bind(task.age)
    .bind(&task.chart_no).bind(&task.bed_no).bind(&task.dept).bind(&task.diagnosis)
    .bind(&task.body_part).bind(&task.procedure).bind(&task.anesthesia).bind(&task.surgeon)
    .bind(&task.vs_note).bind(&task.expected_room).bind(urgency_str(&task.urgency))
    .bind(task.scheduled_at).bind(task.est_time_mins as i64)
    .bind(&task.status).bind(task.id)
    .execute(pool).await.map_err(|e| e.to_string())?;

    fetch_one(pool, task.id).await
}

pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), String> {
    sqlx::query("DELETE FROM surgery_tasks WHERE id=?")
        .bind(id).execute(pool).await.map_err(|e| e.to_string())?;
    Ok(())
}

/// Replaces all tasks in a single transaction (used for cloud pull).
pub async fn replace_all(pool: &SqlitePool, tasks: &[SurgeryTask]) -> Result<(), String> {
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM surgery_tasks")
        .execute(&mut *tx).await.map_err(|e| e.to_string())?;
    for t in tasks {
        sqlx::query(
            "INSERT INTO surgery_tasks
             (id,seq_no,patient_name,gender,age,chart_no,bed_no,dept,diagnosis,body_part,procedure,anesthesia,surgeon,vs_note,expected_room,urgency,scheduled_at,created_at,est_time_mins,status)
             VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
        )
        .bind(t.id).bind(t.seq_no).bind(&t.patient_name).bind(&t.gender).bind(t.age)
        .bind(&t.chart_no).bind(&t.bed_no).bind(&t.dept).bind(&t.diagnosis)
        .bind(&t.body_part).bind(&t.procedure).bind(&t.anesthesia).bind(&t.surgeon)
        .bind(&t.vs_note).bind(&t.expected_room).bind(urgency_str(&t.urgency))
        .bind(t.scheduled_at).bind(t.created_at)
        .bind(t.est_time_mins as i64).bind(&t.status)
        .execute(&mut *tx).await.map_err(|e| e.to_string())?;
    }
    tx.commit().await.map_err(|e| e.to_string())
}

pub async fn batch_create(pool: &SqlitePool, tasks: &[SurgeryTask]) -> Result<(), String> {
    if tasks.is_empty() { return Ok(()); }
    let now = now_secs();
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    for t in tasks {
        sqlx::query(
            "INSERT INTO surgery_tasks
             (seq_no,patient_name,gender,age,chart_no,bed_no,dept,diagnosis,body_part,procedure,
              anesthesia,surgeon,vs_note,expected_room,urgency,scheduled_at,created_at,est_time_mins,status)
             VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
        )
        .bind(t.seq_no).bind(&t.patient_name).bind(&t.gender).bind(t.age)
        .bind(&t.chart_no).bind(&t.bed_no).bind(&t.dept).bind(&t.diagnosis)
        .bind(&t.body_part).bind(&t.procedure).bind(&t.anesthesia).bind(&t.surgeon)
        .bind(&t.vs_note).bind(&t.expected_room).bind(urgency_str(&t.urgency))
        .bind(t.scheduled_at).bind(now)
        .bind(t.est_time_mins as i64).bind(&t.status)
        .execute(&mut *tx).await.map_err(|e| e.to_string())?;
    }
    tx.commit().await.map_err(|e| e.to_string())
}

async fn fetch_one(pool: &SqlitePool, id: i64) -> Result<SurgeryTask, String> {
    sqlx::query("SELECT * FROM surgery_tasks WHERE id=?")
        .bind(id).fetch_one(pool).await
        .map(|r| map_row(&r))
        .map_err(|e| e.to_string())
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
