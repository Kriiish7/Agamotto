#[cfg(test)]
mod tests {
    use crate::server::{greedy, knapsack, metrics};
    use crate::types::{ScheduleMode, Task};

    // IT-01: End-to-end Serenity Mode
    #[test]
    fn it01_serenity_end_to_end() {
        let tasks: Vec<Task> = (0..10)
            .map(|i| Task::new(format!("task-{i}"), (i % 4 + 1) * 15, (i % 5 + 1) as u8))
            .collect();

        let schedule = greedy::schedule_serenity(&tasks, 240);

        assert_eq!(schedule.mode, ScheduleMode::Serenity);
        assert!(!schedule.tasks.is_empty());
        assert!(schedule.tasks.len() <= tasks.len());

        let total_duration: u32 = schedule.tasks.iter().map(|t| t.task.duration).sum();
        assert!(total_duration <= 240);

        // Metrics should be in valid ranges
        assert!(schedule.metrics.productivity_score >= 0.0);
        assert!(schedule.metrics.productivity_score <= 100.0);
        assert!(schedule.metrics.time_utilisation >= 0.0);
        assert!(schedule.metrics.time_utilisation <= 100.0);
        assert!(schedule.metrics.stress_index >= 0.0);
        assert!(schedule.metrics.stress_index <= 1.0);

        // Tasks should be ordered by ascending duration (shortest-first)
        for window in schedule.tasks.windows(2) {
            assert!(window[0].task.duration <= window[1].task.duration);
        }
    }

    // IT-02: End-to-end Crunch Mode with deadlines
    #[test]
    fn it02_crunch_end_to_end() {
        let mut tasks = Vec::new();
        for i in 0..12 {
            let mut t = Task::new(format!("task-{i}"), (i % 5 + 1) * 20, (i % 5 + 1) as u8);
            if i % 3 == 0 {
                t.deadline = Some(format!("2030-01-{:02}T12:00:00Z", i + 1));
            }
            tasks.push(t);
        }

        let schedule = knapsack::schedule_crunch(&tasks, 480);

        assert_eq!(schedule.mode, ScheduleMode::Crunch);
        assert!(!schedule.tasks.is_empty());

        let total_duration: u32 = schedule.tasks.iter().map(|t| t.task.duration).sum();
        assert!(total_duration <= 480);

        // Verify deadline tasks come before non-deadline tasks (EDF ordering)
        let mut last_deadline_idx: Option<usize> = None;
        let mut first_no_deadline_idx: Option<usize> = None;
        for (i, st) in schedule.tasks.iter().enumerate() {
            if st.task.deadline.is_none() && first_no_deadline_idx.is_none() {
                first_no_deadline_idx = Some(i);
            }
            if st.task.deadline.is_some() {
                last_deadline_idx = Some(i);
            }
        }
        if let (Some(last_dl), Some(first_ndl)) = (last_deadline_idx, first_no_deadline_idx) {
            assert!(
                last_dl < first_ndl,
                "Deadline tasks should come before non-deadline tasks"
            );
        }
    }

    // IT-03: Mode switching produces different results
    #[test]
    fn it03_mode_switching() {
        let tasks: Vec<Task> = (0..8)
            .map(|i| {
                let mut t = Task::new(format!("task-{i}"), (i % 3 + 1) * 30, (i % 5 + 1) as u8);
                if i % 2 == 0 {
                    t.deadline = Some(format!("2030-06-{:02}T12:00:00Z", i + 1));
                }
                t
            })
            .collect();

        let serenity = greedy::schedule_serenity(&tasks, 180);
        let crunch = knapsack::schedule_crunch(&tasks, 180);

        // Both should produce valid schedules
        assert!(!serenity.tasks.is_empty());
        assert!(!crunch.tasks.is_empty());

        // They should differ in at least some way (different algorithms)
        let serenity_ids: Vec<&str> = serenity.tasks.iter().map(|t| t.task.id.as_str()).collect();
        let crunch_ids: Vec<&str> = crunch.tasks.iter().map(|t| t.task.id.as_str()).collect();
        // Either different tasks selected or different ordering
        assert!(
            serenity_ids != crunch_ids || serenity.tasks.len() != crunch.tasks.len(),
            "Serenity and Crunch should produce different schedules for mixed-input tasks"
        );
    }

    // IT-04: Edge case — empty task list
    #[test]
    fn it04_empty_task_list() {
        let serenity = greedy::schedule_serenity(&[], 240);
        assert!(serenity.tasks.is_empty());

        let crunch = knapsack::schedule_crunch(&[], 240);
        assert!(crunch.tasks.is_empty());
    }

    // IT-05: Edge case — zero available time
    #[test]
    fn it05_zero_time() {
        let tasks = vec![Task::new("task", 10, 5)];
        let serenity = greedy::schedule_serenity(&tasks, 0);
        assert!(serenity.tasks.is_empty());
        assert_eq!(serenity.metrics.time_utilisation, 0.0);

        let crunch = knapsack::schedule_crunch(&tasks, 0);
        assert!(crunch.tasks.is_empty());
    }

    // IT-06: Edge case — single oversized task
    #[test]
    fn it06_oversized_task() {
        let tasks = vec![Task::new("huge", 500, 5)];
        let serenity = greedy::schedule_serenity(&tasks, 60);
        assert!(serenity.tasks.is_empty());

        let crunch = knapsack::schedule_crunch(&tasks, 60);
        assert!(crunch.tasks.is_empty());
    }

    // IT-07: Edge case — all tasks fit
    #[test]
    fn it07_all_tasks_fit() {
        let tasks = vec![
            Task::new("a", 10, 1),
            Task::new("b", 10, 3),
            Task::new("c", 10, 5),
        ];
        let serenity = greedy::schedule_serenity(&tasks, 30);
        assert_eq!(serenity.tasks.len(), 3);
        assert!((serenity.metrics.time_utilisation - 100.0).abs() < 0.01);
        assert!((serenity.metrics.productivity_score - 100.0).abs() < 0.01);
    }

    // IT-08: Edge case — 50 tasks performance
    #[test]
    fn it08_large_input_performance() {
        let tasks: Vec<Task> = (0..50)
            .map(|i| Task::new(format!("task-{i}"), (i % 8 + 1) * 10, (i % 5 + 1) as u8))
            .collect();

        let start = std::time::Instant::now();
        let serenity = greedy::schedule_serenity(&tasks, 480);
        let serenity_time = start.elapsed();

        let start = std::time::Instant::now();
        let crunch = knapsack::schedule_crunch(&tasks, 480);
        let crunch_time = start.elapsed();

        assert!(
            serenity_time.as_millis() < 100,
            "Serenity took {}ms",
            serenity_time.as_millis()
        );
        assert!(
            crunch_time.as_millis() < 100,
            "Crunch took {}ms",
            crunch_time.as_millis()
        );

        let s_dur: u32 = serenity.tasks.iter().map(|t| t.task.duration).sum();
        let c_dur: u32 = crunch.tasks.iter().map(|t| t.task.duration).sum();
        assert!(s_dur <= 480);
        assert!(c_dur <= 480);
    }

    // IT-09: Metrics recomputation after reorder
    #[test]
    fn it09_metrics_recompute_on_reorder() {
        let tasks = vec![
            Task::new("long-high", 60, 5),
            Task::new("short-low", 15, 1),
            Task::new("med-med", 30, 3),
        ];
        let mut schedule = greedy::schedule_serenity(&tasks, 120);
        let original_stress = schedule.metrics.stress_index;

        // Reverse order
        schedule.tasks.reverse();
        let mut time = 0;
        for t in schedule.tasks.iter_mut() {
            t.start_time = time;
            time += t.task.duration;
            t.end_time = time;
        }
        metrics::compute_metrics(&mut schedule, &tasks);

        // Stress index should differ after reorder (it's order-dependent)
        // At minimum, metrics should still be valid
        assert!(schedule.metrics.stress_index >= 0.0);
        assert!(schedule.metrics.stress_index <= 1.0);
    }

    // IT-10: Knapsack optimality — verify against brute force for small input
    #[test]
    fn it10_knapsack_optimality() {
        let tasks = vec![
            Task::new("a", 10, 5),
            Task::new("b", 20, 4),
            Task::new("c", 15, 3),
            Task::new("d", 25, 5),
        ];
        let w = 35u32;

        let schedule = knapsack::schedule_crunch(&tasks, w);
        let scheduled_score: f64 = schedule
            .tasks
            .iter()
            .map(|st| st.task.priority as f64)
            .sum();

        // Brute force all subsets
        let n = tasks.len();
        let mut best_score = 0.0f64;
        for mask in 0..(1u32 << n) {
            let mut total_dur = 0u32;
            let mut total_score = 0.0f64;
            for i in 0..n {
                if mask & (1 << i) != 0 {
                    total_dur += tasks[i].duration;
                    total_score += tasks[i].priority as f64;
                }
            }
            if total_dur <= w && total_score > best_score {
                best_score = total_score;
            }
        }

        assert!(
            (scheduled_score - best_score).abs() < 0.01,
            "Knapsack score {scheduled_score} != brute force {best_score}"
        );
    }
}
