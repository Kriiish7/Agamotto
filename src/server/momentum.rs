use crate::types::ScheduledTask;

pub fn apply_momentum_ordering(tasks: &mut Vec<ScheduledTask>) -> f64 {
    let n = tasks.len();
    if n < 3 {
        return 1.0;
    }

    let mut scored: Vec<(usize, f64)> = tasks
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let score = t.task.priority as f64
                / ((t.task.emotional_weight * 0.5 + 0.5) * t.task.duration as f64 / 60.0);
            (i, score)
        })
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    let warmup_count = (n / 4).clamp(1, 3);
    let warmup_indices: Vec<usize> = scored.iter().take(warmup_count).map(|(i, _)| *i).collect();

    let warmup: Vec<ScheduledTask> = warmup_indices.iter().map(|&i| tasks[i].clone()).collect();
    let rest: Vec<ScheduledTask> = tasks
        .iter()
        .enumerate()
        .filter(|(i, _)| !warmup_indices.contains(i))
        .map(|(_, t)| t.clone())
        .collect();

    tasks.clear();
    tasks.extend(warmup);
    tasks.extend(rest);

    let mut time = 0;
    for t in tasks.iter_mut() {
        t.start_time = time;
        time += t.task.duration;
        t.end_time = time;
    }

    let avg_emotional_all: f64 =
        tasks.iter().map(|t| t.task.emotional_weight).sum::<f64>() / n as f64;
    let avg_emotional_first: f64 = tasks
        .iter()
        .take(warmup_count)
        .map(|t| t.task.emotional_weight)
        .sum::<f64>()
        / warmup_count as f64;

    (1.0 - (avg_emotional_first / avg_emotional_all.max(0.01))).clamp(0.0, 1.0)
}
