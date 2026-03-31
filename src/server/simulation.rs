use crate::types::{Schedule, ScheduleMode, Task};

#[derive(Clone, Debug)]
pub enum ScheduleChange {
    RemoveTask(String),
    DelayTask(String),
}

#[derive(Clone, Debug)]
pub struct MetricsDiff {
    pub productivity: f64,
    pub time_utilisation: f64,
    pub stress_index: f64,
    pub deadline_risk: f64,
    pub tasks_scheduled: i32,
}

#[derive(Clone, Debug)]
pub struct SimulationResult {
    pub new_schedule: Schedule,
    pub diff: MetricsDiff,
    pub summary: String,
}

pub fn simulate_change(
    original_tasks: &[Task],
    available_time: u32,
    mode: &ScheduleMode,
    change: &ScheduleChange,
) -> SimulationResult {
    let modified_tasks: Vec<Task> = match change {
        ScheduleChange::RemoveTask(id) => original_tasks
            .iter()
            .filter(|t| t.id != *id)
            .cloned()
            .collect(),
        ScheduleChange::DelayTask(id) => original_tasks
            .iter()
            .map(|t| {
                if t.id == *id {
                    let mut t = t.clone();
                    t.deadline = None;
                    t
                } else {
                    t.clone()
                }
            })
            .collect(),
    };

    let new_schedule = match mode {
        ScheduleMode::Serenity => {
            crate::server::greedy::schedule_serenity(&modified_tasks, available_time)
        }
        ScheduleMode::Crunch => {
            crate::server::knapsack::schedule_crunch(&modified_tasks, available_time)
        }
    };

    let original_schedule = match mode {
        ScheduleMode::Serenity => {
            crate::server::greedy::schedule_serenity(original_tasks, available_time)
        }
        ScheduleMode::Crunch => {
            crate::server::knapsack::schedule_crunch(original_tasks, available_time)
        }
    };

    let diff = MetricsDiff {
        productivity: new_schedule.metrics.productivity_score
            - original_schedule.metrics.productivity_score,
        time_utilisation: new_schedule.metrics.time_utilisation
            - original_schedule.metrics.time_utilisation,
        stress_index: new_schedule.metrics.stress_index - original_schedule.metrics.stress_index,
        deadline_risk: new_schedule.metrics.deadline_risk - original_schedule.metrics.deadline_risk,
        tasks_scheduled: new_schedule.tasks.len() as i32 - original_schedule.tasks.len() as i32,
    };

    let summary = generate_summary(change, &diff);

    SimulationResult {
        new_schedule,
        diff,
        summary,
    }
}

fn generate_summary(change: &ScheduleChange, diff: &MetricsDiff) -> String {
    let action = match change {
        ScheduleChange::RemoveTask(_) => "Dropping task",
        ScheduleChange::DelayTask(_) => "Delaying task",
    };

    let mut effects = Vec::new();
    if diff.stress_index.abs() > 0.05 {
        if diff.stress_index < 0.0 {
            effects.push(format!(
                "stress drops {:.0}%",
                diff.stress_index.abs() * 100.0
            ));
        } else {
            effects.push(format!("stress rises {:.0}%", diff.stress_index * 100.0));
        }
    }
    if diff.deadline_risk.abs() > 0.05 {
        if diff.deadline_risk > 0.0 {
            effects.push(format!(
                "deadline risk up {:.0}%",
                diff.deadline_risk * 100.0
            ));
        } else {
            effects.push(format!(
                "deadline risk down {:.0}%",
                diff.deadline_risk.abs() * 100.0
            ));
        }
    }
    if diff.tasks_scheduled != 0 {
        effects.push(format!(
            "{} task(s) {}",
            diff.tasks_scheduled.abs(),
            if diff.tasks_scheduled > 0 {
                "more"
            } else {
                "fewer"
            }
        ));
    }

    if effects.is_empty() {
        format!("{}: no significant change", action)
    } else {
        format!("{}: {}", action, effects.join(", "))
    }
}
