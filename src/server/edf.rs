use crate::types::ScheduledTask;

pub fn order_by_edf(tasks: &mut [ScheduledTask]) {
    tasks.sort_by(|a, b| match (&a.task.deadline, &b.task.deadline) {
        (Some(a_dl), Some(b_dl)) => a_dl.cmp(b_dl),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => b.task.priority.cmp(&a.task.priority),
    });
}
