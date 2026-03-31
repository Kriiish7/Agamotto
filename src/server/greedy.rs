use crate::types::{Schedule, ScheduleMetrics, ScheduleMode, ScheduledTask, Task};

pub fn schedule_serenity(tasks: &[Task], available_time: u32) -> Schedule {
    if tasks.is_empty() {
        return Schedule {
            mode: ScheduleMode::Serenity,
            tasks: Vec::new(),
            available_time,
            metrics: ScheduleMetrics::default(),
        };
    }

    let mut scored: Vec<(usize, f64)> = tasks
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let density = (t.priority as f64 + t.emotional_weight * 2.0) / t.duration.max(1) as f64;
            (i, density)
        })
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    let mut selected: Vec<ScheduledTask> = Vec::new();
    let mut remaining = available_time;

    for (idx, _score) in &scored {
        let task = &tasks[*idx];
        if task.duration <= remaining {
            selected.push(ScheduledTask {
                task: task.clone(),
                start_time: 0,
                end_time: 0,
                stress_contribution: 0.0,
                deadline_risk: 0.0,
            });
            remaining -= task.duration;
        }
    }

    selected.sort_by(|a, b| a.task.duration.cmp(&b.task.duration));

    let mut time = 0;
    for st in &mut selected {
        st.start_time = time;
        time += st.task.duration;
        st.end_time = time;
    }

    let mut schedule = Schedule {
        mode: ScheduleMode::Serenity,
        tasks: selected,
        available_time,
        metrics: ScheduleMetrics::default(),
    };

    super::metrics::compute_metrics(&mut schedule, tasks);
    schedule
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gt01_selects_high_density_tasks() {
        let tasks = vec![
            Task::new("task1", 10, 5),
            Task::new("task2", 20, 3),
            Task::new("task3", 15, 4),
        ];
        let schedule = schedule_serenity(&tasks, 30);
        assert_eq!(schedule.tasks.len(), 2);
        assert_eq!(schedule.tasks[0].task.name, "task1");
        assert_eq!(schedule.tasks[1].task.name, "task3");
    }

    #[test]
    fn gt02_all_tasks_fit() {
        let tasks = vec![
            Task::new("a", 10, 1),
            Task::new("b", 10, 2),
            Task::new("c", 10, 3),
            Task::new("d", 10, 4),
            Task::new("e", 10, 5),
        ];
        let schedule = schedule_serenity(&tasks, 60);
        assert_eq!(schedule.tasks.len(), 5);
        for window in schedule.tasks.windows(2) {
            assert!(window[0].task.duration <= window[1].task.duration);
        }
    }

    #[test]
    fn gt03_oversized_single_task() {
        let tasks = vec![Task::new("big", 500, 5)];
        let schedule = schedule_serenity(&tasks, 60);
        assert!(schedule.tasks.is_empty());
        assert_eq!(schedule.metrics.time_utilisation, 0.0);
    }

    #[test]
    fn gt04_identical_priorities() {
        let tasks = vec![
            Task::new("c", 30, 3),
            Task::new("a", 10, 3),
            Task::new("b", 20, 3),
        ];
        let schedule = schedule_serenity(&tasks, 60);
        assert_eq!(schedule.tasks.len(), 3);
        assert_eq!(schedule.tasks[0].task.name, "a");
        assert_eq!(schedule.tasks[1].task.name, "b");
        assert_eq!(schedule.tasks[2].task.name, "c");
    }

    #[test]
    fn gt05_zero_available_time() {
        let tasks = vec![Task::new("task", 10, 5)];
        let schedule = schedule_serenity(&tasks, 0);
        assert!(schedule.tasks.is_empty());
    }
}
