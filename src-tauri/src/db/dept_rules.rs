use sqlx::{Row, SqlitePool};
use crate::models::DeptRule;

fn map_row(r: &sqlx::sqlite::SqliteRow) -> DeptRule {
    let rooms_json: String = r.get("preferred_rooms");
    DeptRule {
        dept:           r.get("dept"),
        priority_bonus: r.get("priority_bonus"),
        preferred_rooms: serde_json::from_str(&rooms_json).unwrap_or_default(),
        color:          r.get("color"),
    }
}

pub async fn get_all(pool: &SqlitePool) -> Result<Vec<DeptRule>, String> {
    sqlx::query("SELECT * FROM dept_rules ORDER BY dept ASC")
        .fetch_all(pool).await
        .map(|rows| rows.iter().map(map_row).collect())
        .map_err(|e| e.to_string())
}

pub async fn upsert(pool: &SqlitePool, rule: &DeptRule) -> Result<DeptRule, String> {
    let rooms_json = serde_json::to_string(&rule.preferred_rooms)
        .map_err(|e| e.to_string())?;
    sqlx::query(
        "INSERT INTO dept_rules (dept, priority_bonus, preferred_rooms, color)
         VALUES (?, ?, ?, ?)
         ON CONFLICT(dept) DO UPDATE SET priority_bonus=excluded.priority_bonus, preferred_rooms=excluded.preferred_rooms, color=excluded.color",
    )
    .bind(&rule.dept).bind(rule.priority_bonus).bind(&rooms_json).bind(&rule.color)
    .execute(pool).await.map_err(|e| e.to_string())?;

    sqlx::query("SELECT * FROM dept_rules WHERE dept=?")
        .bind(&rule.dept).fetch_one(pool).await
        .map(|r| map_row(&r))
        .map_err(|e| e.to_string())
}

pub async fn delete(pool: &SqlitePool, dept: &str) -> Result<(), String> {
    sqlx::query("DELETE FROM dept_rules WHERE dept=?")
        .bind(dept).execute(pool).await.map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn replace_all(pool: &SqlitePool, rules: &[DeptRule]) -> Result<(), String> {
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM dept_rules")
        .execute(&mut *tx).await.map_err(|e| e.to_string())?;
    for r in rules {
        let rooms = serde_json::to_string(&r.preferred_rooms).map_err(|e| e.to_string())?;
        sqlx::query("INSERT INTO dept_rules (dept, priority_bonus, preferred_rooms, color) VALUES (?,?,?,?)")
            .bind(&r.dept).bind(r.priority_bonus).bind(&rooms).bind(&r.color)
            .execute(&mut *tx).await.map_err(|e| e.to_string())?;
    }
    tx.commit().await.map_err(|e| e.to_string())
}
