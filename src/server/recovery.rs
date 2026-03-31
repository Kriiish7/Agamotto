use crate::types::{Schedule, ScheduledTask, Task};

pub fn insert_micro_recoveries(schedule: &mut Schedule, config: &RecoveryConfig) {
    let mut insertions: Vec<(usize, ScheduledTask)> = Vec::new();
    let mut cumulative_work: u32 = 0;

    for (i, scheduled) in schedule.tasks.iter().enumerate() {
        cumulative_work += scheduled.task.duration;

        if scheduled.task.emotional_weight > config.heavy_threshold {
            insertions.push((i + 1, make_break(config.heavy_break_minutes)));
        }

        if scheduled.task.duration > config.long_task_threshold_minutes {
            insertions.push((i + 1, make_break(config.long_break_minutes)));
        }

        if cumulative_work >= config.cumulative_threshold_minutes {
            insertions.push((i + 1, make_break(config.cumulative_break_minutes)));
            cumulative_work = 0;
        }
    }

    for (pos, break_task) in insertions.into_iter().rev() {
        schedule
            .tasks
            .insert(pos.min(schedule.tasks.len()), break_task);
    }

    let mut time = 0;
    for t in schedule.tasks.iter_mut() {
        t.start_time = time;
        time += t.task.duration;
        t.end_time = time;
    }
}

pub struct RecoveryConfig {
    pub heavy_threshold: f64,
    pub heavy_break_minutes: u32,
    pub long_task_threshold_minutes: u32,
    pub long_break_minutes: u32,
    pub cumulative_threshold_minutes: u32,
    pub cumulative_break_minutes: u32,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            heavy_threshold: 0.6,
            heavy_break_minutes: 5,
            long_task_threshold_minutes: 60,
            long_break_minutes: 5,
            cumulative_threshold_minutes: 120,
            cumulative_break_minutes: 10,
        }
    }
}

fn make_break(minutes: u32) -> ScheduledTask {
    ScheduledTask {
        task: Task {
            id: format!("break-{}", crate::types::uuid_v4()),
            name: match minutes {
                5 => "Quick reset".into(),
                10 => "Take a breath".into(),
                15 => "Real break — walk away".into(),
                _ => "Break".into(),
            },
            duration: minutes,
            priority: 0,
            deadline: None,
            tags: vec!["break".into()],
            emotional_weight: 0.0,
            category: Some("recovery".into()),
        },
        start_time: 0,
        end_time: 0,
        stress_contribution: 0.0,
        deadline_risk: 0.0,
    }
}
