use crate::types::Task;

pub fn compute_decision_debt(tasks: &[Task]) -> u32 {
    let mut debt = 0u32;
    for task in tasks {
        if task.deadline.is_none() {
            debt += 1;
        }
        if task.priority == 3 {
            debt += 1;
        }
        if task.tags.is_empty() {
            debt += 1;
        }
        if task.category.is_none() {
            debt += 1;
        }
        let name_lower = task.name.to_lowercase();
        if task.name.len() < 5
            || name_lower.contains("stuff")
            || name_lower.contains("things")
            || name_lower.contains("work on")
        {
            debt += 1;
        }
    }
    debt
}
