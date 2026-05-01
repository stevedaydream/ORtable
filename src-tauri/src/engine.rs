use crate::models::{DeptRule, ExtraComplianceResult, Staff, StaffComplianceResult, SurgeryTask, TaskWithScore, UrgencyLevel};

// ── Deadline helpers ──────────────────────────────────────────────────────────

/// Hard deadline in seconds from created_at. None = Normal (no deadline).
fn deadline_seconds(urgency: &UrgencyLevel) -> Option<i64> {
    match urgency {
        UrgencyLevel::Trauma => Some(30 * 60),
        UrgencyLevel::Level1 => Some(2 * 3600),
        UrgencyLevel::Level2 => Some(6 * 3600),
        UrgencyLevel::Level3 => Some(24 * 3600),
        UrgencyLevel::Normal => None,
    }
}

/// Deadline elapsed percentage (0.0–1.0+). >1.0 means overdue.
pub fn deadline_elapsed(urgency: &UrgencyLevel, created_at: i64, now: i64) -> Option<f64> {
    deadline_seconds(urgency).map(|d| (now - created_at) as f64 / d as f64)
}

// ── Scoring components ────────────────────────────────────────────────────────

/// Dynamic bonus: grows from 0 to base_weight/2 as deadline is approached.
/// For Normal tasks: +1 per minute waiting (small steady increment).
fn dynamic_bonus(urgency: &UrgencyLevel, created_at: i64, now: i64) -> i64 {
    let elapsed = (now - created_at).max(0);
    match deadline_seconds(urgency) {
        Some(deadline) => {
            let ratio = (elapsed as f64 / deadline as f64).min(1.0);
            (urgency.base_weight() as f64 * ratio * 0.5) as i64
        }
        None => elapsed / 60,
    }
}

/// +5000 if the task's expected room is currently available.
fn room_bonus(task: &SurgeryTask, available_rooms: &[String]) -> i64 {
    if available_rooms.iter().any(|r| r == &task.expected_room) {
        5_000
    } else {
        0
    }
}

/// Dept-configured custom bonus (e.g. a dept may bump its own cases).
fn dept_bonus(task: &SurgeryTask, dept_rules: &[DeptRule]) -> i64 {
    dept_rules
        .iter()
        .find(|r| r.dept == task.dept)
        .map(|r| r.priority_bonus)
        .unwrap_or(0)
}

/// Scheduled time bonus: +3000 base, decreasing by 1 per minute in the future.
/// Unscheduled tasks get 0.
fn scheduled_bonus(task: &SurgeryTask, now: i64) -> i64 {
    match task.scheduled_at {
        Some(sched) => {
            let future_mins = (sched - now).max(0) / 60;
            (3_000 - future_mins).max(0)
        }
        None => 0,
    }
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Full composite priority score (higher = more urgent).
pub fn priority_score(
    task: &SurgeryTask,
    now: i64,
    dept_rules: &[DeptRule],
    available_rooms: &[String],
) -> i64 {
    task.urgency.base_weight()
        + dynamic_bonus(&task.urgency, task.created_at, now)
        + room_bonus(task, available_rooms)
        + dept_bonus(task, dept_rules)
        + scheduled_bonus(task, now)
}

/// Sorts tasks and returns each paired with its score and deadline metadata.
/// Primary sort: score desc. Tie-breaker: created_at asc (FIFO).
pub fn sort_tasks(
    tasks: Vec<SurgeryTask>,
    now: i64,
    dept_rules: &[DeptRule],
    available_rooms: &[String],
) -> Vec<TaskWithScore> {
    let mut scored: Vec<TaskWithScore> = tasks
        .into_iter()
        .map(|t| {
            let score = priority_score(&t, now, dept_rules, available_rooms);
            let elapsed = deadline_elapsed(&t.urgency, t.created_at, now);
            TaskWithScore {
                score,
                is_overdue: elapsed.map(|e| e >= 1.0).unwrap_or(false),
                deadline_elapsed_pct: elapsed.map(|e| (e * 100.0).min(100.0)),
                task: t,
            }
        })
        .collect();

    scored.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then(a.task.created_at.cmp(&b.task.created_at))
    });

    scored
}

// ── Extra line compliance ─────────────────────────────────────────────────────

const TWELVE_HOURS: i64 = 12 * 3600;

/// Single-staff compliance check with precise hour/minute messaging.
pub fn check_compliance(
    today_shift_start: i64,
    next_day_shift_start: i64,
    estimated_extra_end: i64,
) -> ExtraComplianceResult {
    let work_secs = estimated_extra_end - today_shift_start;
    if work_secs > TWELVE_HOURS {
        let h = work_secs / 3600;
        let m = (work_secs % 3600) / 60;
        return ExtraComplianceResult {
            allowed: false,
            reason: Some(format!("當日工時 {h}h{m:02}m 超過 12 小時上限")),
        };
    }

    let rest_secs = next_day_shift_start - estimated_extra_end;
    if rest_secs < TWELVE_HOURS {
        let h = rest_secs / 3600;
        let m = (rest_secs % 3600) / 60;
        return ExtraComplianceResult {
            allowed: false,
            reason: Some(format!("與明日班距僅 {h}h{m:02}m，不足 12 小時")),
        };
    }

    ExtraComplianceResult { allowed: true, reason: None }
}

/// Batch compliance check — returns result for every staff member.
pub fn batch_compliance(
    staff_list: &[Staff],
    estimated_extra_end: i64,
) -> Vec<StaffComplianceResult> {
    staff_list
        .iter()
        .map(|s| {
            let result =
                check_compliance(s.today_shift_start, s.next_day_shift_start, estimated_extra_end);
            StaffComplianceResult {
                staff_id: s.id,
                allowed: result.allowed,
                reason: result.reason,
            }
        })
        .collect()
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trauma_overdue_when_past_30_min() {
        let task = SurgeryTask {
            id: 1,
            urgency: UrgencyLevel::Trauma,
            created_at: 0,
            ..Default::default()
        };
        let elapsed = deadline_elapsed(&task.urgency, task.created_at, 31 * 60);
        assert!(elapsed.unwrap() > 1.0);
    }

    #[test]
    fn compliance_blocks_on_work_hour_exceed() {
        let result = check_compliance(0, 50 * 3600, 13 * 3600);
        assert!(!result.allowed);
        assert!(result.reason.unwrap().contains("工時"));
    }

    #[test]
    fn compliance_blocks_on_rest_period_short() {
        let result = check_compliance(0, 20 * 3600, 12 * 3600);
        assert!(!result.allowed);
        assert!(result.reason.unwrap().contains("班距"));
    }

    #[test]
    fn compliance_allows_when_within_limits() {
        // 8h shift, 12h extra end, next shift at 26h → rest = 14h
        let result = check_compliance(0, 26 * 3600, 12 * 3600);
        assert!(result.allowed);
    }

    #[test]
    fn sort_trauma_before_normal() {
        let now = 1_000_000i64;
        let tasks = vec![
            SurgeryTask { id: 1, urgency: UrgencyLevel::Normal, created_at: now, ..Default::default() },
            SurgeryTask { id: 2, urgency: UrgencyLevel::Trauma, created_at: now, ..Default::default() },
        ];
        let sorted = sort_tasks(tasks, now, &[], &[]);
        assert_eq!(sorted[0].task.id, 2);
    }
}
