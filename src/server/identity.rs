use crate::types::ScheduledTask;
use std::collections::HashSet;

pub fn detect_identity_conflicts(tasks: &[ScheduledTask]) -> Vec<String> {
    let mut categories: Vec<&str> = tasks
        .iter()
        .filter_map(|t| t.task.category.as_deref())
        .collect();

    let distinct: HashSet<&str> = categories.iter().copied().collect();
    let mut conflicts = Vec::new();

    if distinct.len() >= 4 {
        let cats: Vec<&str> = distinct.iter().copied().collect();
        conflicts.push(format!(
            "You're wearing {} hats today: {}. Consider focusing on 2-3 roles max.",
            distinct.len(),
            cats.join(", ")
        ));
    }

    let high_pri_by_cat: std::collections::HashMap<&str, usize> = tasks
        .iter()
        .filter(|t| t.task.priority >= 4)
        .filter_map(|t| t.task.category.as_deref().map(|c| (c, t)))
        .fold(std::collections::HashMap::new(), |mut acc, (cat, _)| {
            *acc.entry(cat).or_insert(0) += 1;
            acc
        });

    let high_pri_cats: Vec<&&str> = high_pri_by_cat.keys().collect();
    for combo in high_pri_cats.windows(2) {
        conflicts.push(format!(
            "Conflicting identities: your '{}' tasks and '{}' tasks are both competing for your best energy.",
            combo[0], combo[1]
        ));
    }

    let mut switches = 0;
    for window in tasks.windows(2) {
        let cat_a = window[0]
            .task
            .category
            .as_deref()
            .unwrap_or("uncategorised");
        let cat_b = window[1]
            .task
            .category
            .as_deref()
            .unwrap_or("uncategorised");
        if cat_a != cat_b {
            switches += 1;
        }
    }
    let switch_rate = switches as f64 / (tasks.len().saturating_sub(1)).max(1) as f64;
    if switch_rate > 0.6 {
        conflicts.push(format!(
            "You're switching between roles every {:.1} tasks. That's a lot of context switching.",
            1.0 / switch_rate
        ));
    }

    conflicts
}
