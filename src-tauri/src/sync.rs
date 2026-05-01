use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use crate::{db, models::{DeptRule, Staff, SurgeryTask}};

// ── Wire types ────────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct PushPayload {
    action: &'static str,
    tasks: Vec<SurgeryTask>,
    staff: Vec<Staff>,
    dept_rules: Vec<DeptRule>,
}

#[derive(Deserialize)]
struct PushResponse {
    ok: Option<bool>,
    error: Option<String>,
    synced_at: Option<i64>,
}

#[derive(Deserialize)]
struct PullTasksResponse {
    tasks: Option<Vec<SurgeryTask>>,
    error: Option<String>,
}

#[derive(Deserialize)]
struct PullStaffResponse {
    staff: Option<Vec<Staff>>,
    error: Option<String>,
}

#[derive(Deserialize)]
struct PullRulesResponse {
    dept_rules: Option<Vec<DeptRule>>,
    error: Option<String>,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Reads all local data and POSTs it to GAS. Returns a status message.
pub async fn push(pool: &SqlitePool, gas_url: &str) -> Result<String, String> {
    let tasks = db::tasks::get_all(pool).await?;
    let staff = db::staff::get_all(pool).await?;
    let dept_rules = db::dept_rules::get_all(pool).await?;

    let task_count = tasks.len();
    let staff_count = staff.len();

    let payload = PushPayload {
        action: "syncAll",
        tasks,
        staff,
        dept_rules,
    };

    let client = build_client()?;
    let resp: PushResponse = client
        .post(gas_url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("網路錯誤: {e}"))?
        .json()
        .await
        .map_err(|e| format!("回應解析錯誤: {e}"))?;

    if let Some(err) = resp.error {
        return Err(format!("GAS 錯誤: {err}"));
    }

    // Record last push time
    let now = now_secs();
    db::settings::set(pool, "last_push_at", &now.to_string()).await?;

    Ok(format!(
        "推送成功：{task_count} 筆手術、{staff_count} 位人員"
    ))
}

/// Pulls all data from GAS and replaces local SQLite content.
pub async fn pull(pool: &SqlitePool, gas_url: &str) -> Result<String, String> {
    let client = build_client()?;

    // Fetch each resource in parallel
    let (tasks_res, staff_res, rules_res) = tokio::try_join!(
        fetch_json::<PullTasksResponse>(&client, gas_url, "getTasks"),
        fetch_json::<PullStaffResponse>(&client, gas_url, "getStaff"),
        fetch_json::<PullRulesResponse>(&client, gas_url, "getDeptRules"),
    )
    .map_err(|e| e.to_string())?;

    if let Some(e) = tasks_res.error.or(staff_res.error).or(rules_res.error) {
        return Err(format!("GAS 錯誤: {e}"));
    }

    let tasks = tasks_res.tasks.unwrap_or_default();
    let staff = staff_res.staff.unwrap_or_default();
    let rules = rules_res.dept_rules.unwrap_or_default();

    let task_count = tasks.len();
    let staff_count = staff.len();

    db::tasks::replace_all(pool, &tasks).await?;
    db::staff::replace_all(pool, &staff).await?;
    db::dept_rules::replace_all(pool, &rules).await?;

    let now = now_secs();
    db::settings::set(pool, "last_pull_at", &now.to_string()).await?;

    Ok(format!(
        "拉取成功：{task_count} 筆手術、{staff_count} 位人員"
    ))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn build_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())
}

async fn fetch_json<T: serde::de::DeserializeOwned>(
    client: &reqwest::Client,
    gas_url: &str,
    action: &str,
) -> Result<T, String> {
    client
        .get(format!("{gas_url}?action={action}"))
        .send()
        .await
        .map_err(|e| format!("網路錯誤({action}): {e}"))?
        .json::<T>()
        .await
        .map_err(|e| format!("解析錯誤({action}): {e}"))
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
