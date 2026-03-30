use crate::state;
use dioxus::prelude::*;

#[component]
pub fn ExcludedList() -> Element {
    let schedule = state::SCHEDULE.read();
    let all_tasks = state::TASKS.read();

    let (schedule_ref, _) = match schedule.as_ref() {
        Some(s) => (s, ()),
        None => return rsx! { div {} },
    };

    let scheduled_ids: std::collections::HashSet<&str> = schedule_ref
        .tasks
        .iter()
        .map(|t| t.task.id.as_str())
        .collect();

    let excluded: Vec<_> = all_tasks
        .iter()
        .filter(|t| !scheduled_ids.contains(t.id.as_str()))
        .collect();

    if excluded.is_empty() {
        return rsx! { div {} };
    }

    let scheduled_duration: u32 = schedule_ref.tasks.iter().map(|t| t.task.duration).sum();
    let remaining = schedule_ref
        .available_time
        .saturating_sub(scheduled_duration);

    rsx! {
        div { class: "excluded-list",
            h3 { "Excluded Tasks ({excluded.len()})" }
            for task in &excluded {
                div { key: "{task.id}", class: "excluded-row",
                    span { class: "excluded-name", "{task.name}" }
                    span { class: "excluded-reason",
                        if task.duration > remaining {
                            "Needs {task.duration} min, only {remaining} min remaining"
                        } else {
                            "Lower priority displaced by higher-priority tasks"
                        }
                    }
                }
            }
        }
    }
}
