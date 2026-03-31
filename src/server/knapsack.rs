use crate::types::{Schedule, ScheduleMetrics, ScheduleMode, ScheduledTask, Task};

pub fn schedule_crunch(tasks: &[Task], available_time: u32) -> Schedule {
    if tasks.is_empty() {
        return Schedule {
            mode: ScheduleMode::Crunch,
            tasks: Vec::new(),
            available_time,
            metrics: ScheduleMetrics::default(),
        };
    }

    let n = tasks.len();
    let w = available_time as usize;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as f64;

    let crunch_scores: Vec<f64> = tasks
        .iter()
        .map(|t| {
            let urgency = compute_urgency(t, now);
            t.priority as f64 * (1.0 + urgency) * (1.0 + t.emotional_weight * 0.5)
        })
        .collect();

    let mut dp = vec![vec![0.0f64; w + 1]; n + 1];

    for i in 1..=n {
        let duration = tasks[i - 1].duration as usize;
        let score = crunch_scores[i - 1];
        for j in 0..=w {
            dp[i][j] = dp[i - 1][j];
            if duration <= j {
                let with = dp[i - 1][j - duration] + score;
                if with > dp[i][j] {
                    dp[i][j] = with;
                }
            }
        }
    }

    let mut selected_indices = Vec::new();
    let mut remaining = w;
    for i in (1..=n).rev() {
        if dp[i][remaining] != dp[i - 1][remaining] {
            selected_indices.push(i - 1);
            remaining -= tasks[i - 1].duration as usize;
        }
    }
    selected_indices.reverse();

    let mut selected: Vec<ScheduledTask> = selected_indices
        .iter()
        .map(|&idx| ScheduledTask {
            task: tasks[idx].clone(),
            start_time: 0,
            end_time: 0,
            stress_contribution: 0.0,
            deadline_risk: 0.0,
        })
        .collect();

    super::edf::order_by_edf(&mut selected);

    let mut time = 0;
    for st in &mut selected {
        st.start_time = time;
        time += st.task.duration;
        st.end_time = time;
    }

    let mut schedule = Schedule {
        mode: ScheduleMode::Crunch,
        tasks: selected,
        available_time,
        metrics: ScheduleMetrics::default(),
    };

    super::metrics::compute_metrics(&mut schedule, tasks);
    schedule
}

fn compute_urgency(task: &Task, now_secs: f64) -> f64 {
    if let Some(deadline_str) = &task.deadline {
        if let Ok(deadline) = parse_iso_to_secs(deadline_str) {
            let hours_until = (deadline - now_secs) / 3600.0;
            if hours_until <= 0.0 {
                return 10.0;
            }
            return 1.0 / hours_until.max(1.0);
        }
    }
    0.0
}

fn parse_iso_to_secs(iso: &str) -> Result<f64, ()> {
    let trimmed = iso.trim();
    // Try "YYYY-MM-DDTHH:MM:SSZ" or "YYYY-MM-DDTHH:MM:SS+00:00"
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

    // Parse "YYYY-MM-DDTHH:MM:SS" -> seconds since epoch (approximate)
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

        // Days since epoch (simplified, ignores leap seconds)
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

    #[test]
    fn kt01_optimal_subset() {
        let tasks = vec![
            Task::new("task1", 10, 5),
            Task::new("task2", 20, 4),
            Task::new("task3", 15, 3),
            Task::new("task4", 25, 5),
        ];
        let schedule = schedule_crunch(&tasks, 35);
        let total_dur: u32 = schedule.tasks.iter().map(|t| t.task.duration).sum();
        assert!(total_dur <= 35);
        assert!(!schedule.tasks.is_empty());
    }

    #[test]
    fn kt02_large_input_performance() {
        let tasks: Vec<Task> = (0..50)
            .map(|i| Task::new(format!("task{i}"), (i % 8 + 1) * 10, (i % 5 + 1) as u8))
            .collect();
        let start = std::time::Instant::now();
        let schedule = schedule_crunch(&tasks, 480);
        let elapsed = start.elapsed();
        assert!(elapsed.as_millis() < 100, "Took {}ms", elapsed.as_millis());
        let total_dur: u32 = schedule.tasks.iter().map(|t| t.task.duration).sum();
        assert!(total_dur <= 480);
    }

    #[test]
    fn kt03_past_deadline() {
        let mut task = Task::new("overdue", 10, 5);
        task.deadline = Some("2020-01-01T00:00:00Z".to_string());
        let schedule = schedule_crunch(&[task], 60);
        assert!(!schedule.tasks.is_empty());
    }

    #[test]
    fn kt04_identical_scores() {
        let tasks = vec![
            Task::new("a", 10, 3),
            Task::new("b", 10, 3),
            Task::new("c", 10, 3),
        ];
        let schedule = schedule_crunch(&tasks, 30);
        assert_eq!(schedule.tasks.len(), 3);
    }

    #[test]
    fn kt05_empty_list() {
        let schedule = schedule_crunch(&[], 60);
        assert!(schedule.tasks.is_empty());
    }
}
