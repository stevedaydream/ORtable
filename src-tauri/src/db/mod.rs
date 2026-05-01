use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use tauri::{AppHandle, Manager};

pub mod dept_rules;
pub mod room_shifts;
pub mod rooms;
pub mod settings;
pub mod staff;
pub mod tasks;

pub async fn init(app: &AppHandle) -> Result<SqlitePool, String> {
    let db_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&db_dir).map_err(|e| e.to_string())?;

    let db_path = db_dir.join("ortriage.db");
    // SQLite URL requires forward slashes on all platforms
    let url = format!(
        "sqlite:{}?mode=rwc",
        db_path.to_string_lossy().replace('\\', "/")
    );

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .map_err(|e| e.to_string())?;

    migrate(&pool).await?;
    Ok(pool)
}

async fn migrate(pool: &SqlitePool) -> Result<(), String> {
    let stmts = [
        "CREATE TABLE IF NOT EXISTS surgery_tasks (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            patient_name TEXT    NOT NULL DEFAULT '',
            chart_no     TEXT    NOT NULL DEFAULT '',
            procedure    TEXT    NOT NULL DEFAULT '',
            diagnosis    TEXT    NOT NULL DEFAULT '',
            vs_note      TEXT    NOT NULL DEFAULT '',
            dept         TEXT    NOT NULL DEFAULT '',
            expected_room TEXT   NOT NULL DEFAULT '',
            urgency      TEXT    NOT NULL DEFAULT 'Normal',
            scheduled_at INTEGER,
            created_at   INTEGER NOT NULL DEFAULT 0,
            est_time_mins INTEGER NOT NULL DEFAULT 60,
            status       TEXT    NOT NULL DEFAULT 'waiting'
        )",
        "CREATE TABLE IF NOT EXISTS staff (
            id                   INTEGER PRIMARY KEY AUTOINCREMENT,
            name                 TEXT    NOT NULL DEFAULT '',
            role                 TEXT    NOT NULL DEFAULT 'Circ',
            type                 TEXT    NOT NULL DEFAULT 'nur',
            staff_category       TEXT    NOT NULL DEFAULT 'or_nurse',
            unit                 TEXT    NOT NULL DEFAULT '',
            is_on_call           INTEGER NOT NULL DEFAULT 0,
            is_volunteer_extra   INTEGER NOT NULL DEFAULT 0,
            today_shift_start    INTEGER NOT NULL DEFAULT 0,
            next_day_shift_start INTEGER NOT NULL DEFAULT 0
        )",
        "CREATE TABLE IF NOT EXISTS rooms (
            id            INTEGER PRIMARY KEY AUTOINCREMENT,
            name          TEXT    NOT NULL DEFAULT '',
            display_order INTEGER NOT NULL DEFAULT 0,
            is_backup     INTEGER NOT NULL DEFAULT 0
        )",
        "CREATE TABLE IF NOT EXISTS room_shifts (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            room_name  TEXT    NOT NULL DEFAULT '',
            dept       TEXT    NOT NULL DEFAULT '',
            date       TEXT    NOT NULL DEFAULT '',
            start_time INTEGER NOT NULL DEFAULT 0,
            end_time   INTEGER NOT NULL DEFAULT 0,
            notes      TEXT    NOT NULL DEFAULT ''
        )",
        "CREATE TABLE IF NOT EXISTS dept_rules (
            dept            TEXT    PRIMARY KEY,
            priority_bonus  INTEGER NOT NULL DEFAULT 0,
            preferred_rooms TEXT    NOT NULL DEFAULT '[]'
        )",
        "CREATE TABLE IF NOT EXISTS settings (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL DEFAULT ''
        )",
    ];

    for stmt in &stmts {
        sqlx::query(stmt)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
    }
    // Idempotent column additions for existing databases
    for stmt in &[
        "ALTER TABLE staff ADD COLUMN staff_category TEXT NOT NULL DEFAULT 'or_nurse'",
        "ALTER TABLE staff ADD COLUMN unit TEXT NOT NULL DEFAULT ''",
    ] {
        let _ = sqlx::query(stmt).execute(pool).await;
    }
    Ok(())
}
