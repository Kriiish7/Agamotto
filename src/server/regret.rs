use crate::types::{Schedule, ScheduleMode, ScheduledTask, Task};

pub fn compute_regret_scores(
    all_tasks: &[Task],
    schedule: &Schedule,
    available_time: u32,
) -> Vec<RegretItem> {
    let scheduled_ids: std::collections::HashSet<&str> =
        schedule.tasks.iter().map(|t| t.task.id.as_str()).collect();

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as f64;

    all_tasks
        .iter()
        .filter(|t| !scheduled_ids.contains(t.id.as_str()))
        .map(|task| {
            let hours_until = if let Some(deadline_str) = &task.deadline {
                if let Ok(dl_secs) = parse_iso_to_secs(deadline_str) {
                    ((dl_secs - now) / 3600.0).max(1.0)
                } else {
                    72.0
                }
            } else {
                72.0
            };

            let regret = task.priority as f64
                * (1.0 / hours_until)
                * (1.0 + task.emotional_weight);

            RegretItem {
                task_id: task.id.clone(),
                task_name: task.name.clone(),
                regret_score: regret,
                reason: format!(
                    "Excluded despite priority {} — {} hours until deadline, emotional weight {:.1}",
                    task.priority, hours_until as u32, task.emotional_weight
                ),
            }
        })
        .filter(|r| r.regret_score > 0.5)
        .collect()
}

pub fn schedule_regret_minimisation(tasks: &[Task], available_time: u32) -> Schedule {
    let serenity = crate::server::greedy::schedule_serenity(tasks, available_time);
    let crunch = crate::server::knapsack::schedule_crunch(tasks, available_time);

    let serenity_regret = compute_regret_scores(tasks, &serenity, available_time);
    let crunch_regret = compute_regret_scores(tasks, &crunch, available_time);

    let serenity_max = serenity_regret
        .iter()
        .map(|r| r.regret_score)
        .fold(0.0, f64::max);
    let crunch_max = crunch_regret
        .iter()
        .map(|r| r.regret_score)
        .fold(0.0, f64::max);

    let mut chosen = if serenity_max <= crunch_max {
        serenity
    } else {
        crunch
    };

    chosen.mode = ScheduleMode::Serenity;
    chosen
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

#[derive(Clone, Debug)]
pub struct RegretItem {
    pub task_id: String,
    pub task_name: String,
    pub regret_score: f64,
    pub reason: String,
}
