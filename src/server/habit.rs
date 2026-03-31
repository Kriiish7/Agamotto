use crate::types::Task;

#[derive(Clone, Debug)]
pub struct DriftSignal {
    pub kind: String,
    pub magnitude: f64,
    pub message: String,
}

#[derive(Clone, Debug)]
pub struct DriftAlert {
    pub signal: DriftSignal,
}

pub fn detect_habit_drift(entries: &[BehaviourEntry]) -> Option<DriftAlert> {
    if entries.len() < 14 {
        return None;
    }

    let recent = &entries[entries.len() - 7..];
    let baseline = &entries[..entries.len() - 7];

    let mut alerts = Vec::new();

    let baseline_crunch_rate =
        baseline.iter().filter(|e| e.mode_used == "crunch").count() as f64 / baseline.len() as f64;
    let recent_crunch_rate =
        recent.iter().filter(|e| e.mode_used == "crunch").count() as f64 / recent.len() as f64;

    if recent_crunch_rate - baseline_crunch_rate > 0.4 {
        alerts.push(DriftSignal {
            kind: "mode_shift".into(),
            magnitude: recent_crunch_rate - baseline_crunch_rate,
            message: format!(
                "You've switched to Crunch Mode {}% of the time recently (was {}% before). Are you under more pressure?",
                (recent_crunch_rate * 100.0) as u32,
                (baseline_crunch_rate * 100.0) as u32,
            ),
        });
    }

    let baseline_avg_pri = mean(baseline, |e| e.avg_priority);
    let recent_avg_pri = mean(recent, |e| e.avg_priority);

    if recent_avg_pri - baseline_avg_pri > 0.8 {
        alerts.push(DriftSignal {
            kind: "priority_creep".into(),
            magnitude: recent_avg_pri - baseline_avg_pri,
            message: format!(
                "Your average task priority has risen from {:.1} to {:.1}. Everything feels urgent now — that's a warning sign.",
                baseline_avg_pri, recent_avg_pri,
            ),
        });
    }

    let baseline_completion = mean(baseline, |e| e.completion_rate);
    let recent_completion = mean(recent, |e| e.completion_rate);

    if baseline_completion - recent_completion > 0.15 {
        alerts.push(DriftSignal {
            kind: "completion_decline".into(),
            magnitude: baseline_completion - recent_completion,
            message: format!(
                "Your completion rate dropped from {}% to {}%. You might be overloading yourself.",
                (baseline_completion * 100.0) as u32,
                (recent_completion * 100.0) as u32,
            ),
        });
    }

    let baseline_emotional = mean(baseline, |e| e.emotional_weight_avg);
    let recent_emotional = mean(recent, |e| e.emotional_weight_avg);

    if recent_emotional - baseline_emotional > 0.2 {
        alerts.push(DriftSignal {
            kind: "emotional_heaviness".into(),
            magnitude: recent_emotional - baseline_emotional,
            message: format!(
                "Tasks are feeling heavier lately (emotional weight: {:.2} → {:.2}). Consider whether you need a lighter week.",
                baseline_emotional, recent_emotional,
            ),
        });
    }

    alerts
        .into_iter()
        .max_by(|a, b| a.magnitude.partial_cmp(&b.magnitude).unwrap())
        .map(|signal| DriftAlert { signal })
}

pub fn log_behaviour_entry(
    tasks: &[Task],
    schedule_task_count: usize,
    mode: &str,
) -> BehaviourEntry {
    BehaviourEntry {
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .to_string(),
        tasks_planned: tasks.len() as u32,
        tasks_scheduled: schedule_task_count as u32,
        avg_priority: if tasks.is_empty() {
            0.0
        } else {
            tasks.iter().map(|t| t.priority as f64).sum::<f64>() / tasks.len() as f64
        },
        mode_used: mode.to_string(),
        emotional_weight_avg: if tasks.is_empty() {
            0.0
        } else {
            tasks.iter().map(|t| t.emotional_weight).sum::<f64>() / tasks.len() as f64
        },
        completion_rate: 0.0,
    }
}

#[derive(Clone, Debug)]
pub struct BehaviourEntry {
    pub timestamp: String,
    pub tasks_planned: u32,
    pub tasks_scheduled: u32,
    pub avg_priority: f64,
    pub mode_used: String,
    pub emotional_weight_avg: f64,
    pub completion_rate: f64,
}

impl BehaviourEntry {
    pub fn new() -> Self {
        Self {
            timestamp: String::new(),
            tasks_planned: 0,
            tasks_scheduled: 0,
            avg_priority: 0.0,
            mode_used: String::new(),
            emotional_weight_avg: 0.0,
            completion_rate: 0.0,
        }
    }
}

fn mean<T, F>(data: &[T], f: F) -> f64
where
    F: Fn(&T) -> f64,
{
    if data.is_empty() {
        return 0.0;
    }
    data.iter().map(f).sum::<f64>() / data.len() as f64
}
