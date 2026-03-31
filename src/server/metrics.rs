use crate::types::{Schedule, Task};

pub fn compute_metrics(schedule: &mut Schedule, all_tasks: &[Task]) {
    if all_tasks.is_empty() {
        return;
    }

    let scheduled = &mut schedule.tasks;
    let w = schedule.available_time as f64;

    let total_priority_all: f64 = all_tasks.iter().map(|t| t.priority as f64).sum();
    let total_priority_scheduled: f64 = scheduled.iter().map(|t| t.task.priority as f64).sum();
    let total_duration_scheduled: u32 = scheduled.iter().map(|t| t.task.duration).sum();

    schedule.metrics.productivity_score = if total_priority_all > 0.0 {
        (total_priority_scheduled / total_priority_all) * 100.0
    } else {
        0.0
    };

    schedule.metrics.time_utilisation = if w > 0.0 {
        (total_duration_scheduled as f64 / w) * 100.0
    } else {
        0.0
    };

    let max_priority = all_tasks.iter().map(|t| t.priority).max().unwrap_or(1) as f64;
    let n = scheduled.len();
    let stress: f64 = scheduled
        .iter()
        .enumerate()
        .map(|(i, st)| {
            let position_weight = if n > 1 {
                0.5 + (i as f64 / (n - 1) as f64)
            } else {
                1.0
            };
            let priority_weight = st.task.priority as f64 / max_priority;
            let emotional = st.task.emotional_weight;
            st.task.duration as f64 * position_weight * (priority_weight + emotional)
        })
        .sum();
    schedule.metrics.stress_index = if w > 0.0 && max_priority > 0.0 {
        (stress / (w * max_priority)).min(1.0)
    } else {
        0.0
    };

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as f64;

    let mut total_deadline_risk = 0.0;
    let mut tasks_with_deadlines = 0;

    for st in scheduled.iter_mut() {
        if let Some(dl_str) = &st.task.deadline {
            if let Ok(dl_secs) = parse_iso_to_secs(dl_str) {
                tasks_with_deadlines += 1;
                let completion_time = now + st.end_time as f64 * 60.0;
                let slack = (dl_secs - completion_time) / 3600.0;
                st.deadline_risk = if slack < 0.0 {
                    1.0
                } else if slack < 1.0 {
                    0.8
                } else if slack < 6.0 {
                    0.4
                } else {
                    0.1
                };
                total_deadline_risk += st.deadline_risk;
            }
        }
    }

    schedule.metrics.deadline_risk = if tasks_with_deadlines > 0 {
        (total_deadline_risk / tasks_with_deadlines as f64) * 100.0
    } else {
        0.0
    };

    let avg_emotional: f64 = if !scheduled.is_empty() {
        scheduled
            .iter()
            .map(|t| t.task.emotional_weight)
            .sum::<f64>()
            / n as f64
    } else {
        0.0
    };
    schedule.metrics.overload_flag = schedule.metrics.stress_index > 0.65
        && schedule.metrics.time_utilisation > 90.0
        && avg_emotional > 0.5;

    schedule.metrics.decision_debt = super::debt::compute_decision_debt(all_tasks);
    schedule.metrics.identity_conflicts = super::identity::detect_identity_conflicts(scheduled);
    schedule.metrics.failure_points =
        super::forecast::forecast_failure_points(scheduled, schedule.available_time);

    let momentum = super::momentum::apply_momentum_ordering(&mut schedule.tasks);
    schedule.metrics.momentum_score = momentum;
}

fn parse_iso_to_secs(iso: &str) -> Result<f64, ()> {
    let trimmed = iso.trim();
    let (date_time, _) = if let Some(idx) = trimmed.rfind('Z') {
        (&trimmed[..idx], "Z")
    } else if let Some(idx) = trimmed.find('+') {
        (&trimmed[..idx], &trimmed[idx..])
    } else if let Some(idx) = trimmed.rfind('-') {
        if idx > 10 {
            (&trimmed[..idx], &trimmed[idx..])
        } else {
            (trimmed, "")
        }
    } else {
        (trimmed, "")
    };

    if date_time.len() >= 19 {
        let parts: Vec<&str> = date_time.split('T').collect();
        if parts.len() != 2 {
            return Err(());
        }
        let date_parts: Vec<&str> = parts[0].split('-').collect();
        let time_parts: Vec<&str> = parts[1].split(':').collect();
        if date_parts.len() != 3 || time_parts.len() < 2 {
            return Err(());
        }

        let year: i64 = date_parts[0].parse().map_err(|_| ())?;
        let month: i64 = date_parts[1].parse().map_err(|_| ())?;
        let day: i64 = date_parts[2].parse().map_err(|_| ())?;
        let hour: i64 = time_parts[0].parse().map_err(|_| ())?;
        let min: i64 = time_parts[1].parse().map_err(|_| ())?;
        let sec: i64 = if time_parts.len() >= 3 {
            time_parts[2].parse().map_err(|_| ())?
        } else {
            0
        };

        let a = (14 - month) / 12;
        let y = year + 4800 - a;
        let m = month + 12 * a - 3;
        let jdn = day + (153 * m + 2) / 5 + 365 * y + y / 4 - y / 100 + y / 400 - 32045;
        let secs = (jdn - 2440588) * 86400 + hour * 3600 + min * 60 + sec;
        return Ok(secs as f64);
    }

    Err(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ScheduleMetrics, ScheduleMode, ScheduledTask};

    fn make_schedule(tasks: Vec<(u32, u8)>, available: u32) -> Schedule {
        let all_tasks: Vec<Task> = tasks
            .iter()
            .enumerate()
            .map(|(i, (dur, pri))| Task::new(format!("t{i}"), *dur, *pri))
            .collect();
        let scheduled: Vec<ScheduledTask> = tasks
            .iter()
            .enumerate()
            .map(|(i, (dur, pri))| {
                let mut t = Task::new(format!("t{i}"), *dur, *pri);
                t.emotional_weight = 0.0;
                ScheduledTask {
                    task: t,
                    start_time: 0,
                    end_time: *dur,
                    stress_contribution: 0.0,
                    deadline_risk: 0.0,
                }
            })
            .collect();
        let mut schedule = Schedule {
            mode: ScheduleMode::Serenity,
            tasks: scheduled,
            available_time: available,
            metrics: ScheduleMetrics::default(),
        };
        compute_metrics(&mut schedule, &all_tasks);
        schedule
    }

    #[test]
    fn mt01_full_schedule() {
        let s = make_schedule(vec![(30, 5), (30, 3)], 60);
        assert!((s.metrics.time_utilisation - 100.0).abs() < 0.01);
        assert!((s.metrics.productivity_score - 100.0).abs() < 0.01);
    }

    #[test]
    fn mt02_partial_productivity() {
        let all = vec![
            Task::new("a", 10, 5),
            Task::new("b", 10, 2),
            Task::new("c", 10, 2),
        ];
        let scheduled = vec![ScheduledTask {
            task: all[0].clone(),
            start_time: 0,
            end_time: 10,
            stress_contribution: 0.0,
            deadline_risk: 0.0,
        }];
        let mut s = Schedule {
            mode: ScheduleMode::Serenity,
            tasks: scheduled,
            available_time: 30,
            metrics: ScheduleMetrics::default(),
        };
        compute_metrics(&mut s, &all);
        let expected = 5.0 / 9.0 * 100.0;
        assert!((s.metrics.productivity_score - expected).abs() < 0.1);
    }

    #[test]
    fn mt03_stress_index_range() {
        let s = make_schedule(vec![(10, 1)], 480);
        assert!(s.metrics.stress_index >= 0.0);
        assert!(s.metrics.stress_index <= 1.0);
    }

    #[test]
    fn mt04_stress_index_positive() {
        let tasks: Vec<(u32, u8)> = (0..10).map(|_| (48, 5)).collect();
        let s = make_schedule(tasks, 480);
        // 10 high-priority tasks filling the entire window produces positive stress
        assert!(
            s.metrics.stress_index > 0.1,
            "stress was {}",
            s.metrics.stress_index
        );
        assert!(
            s.metrics.stress_index < 1.0,
            "stress was {}",
            s.metrics.stress_index
        );
    }
}
