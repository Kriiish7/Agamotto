use crate::state;
use dioxus::prelude::*;

#[component]
pub fn OverloadBanner() -> Element {
    let schedule = state::SCHEDULE.read();
    let schedule = match schedule.as_ref() {
        Some(s) => s,
        None => return rsx! { div {} },
    };

    if !schedule.metrics.overload_flag {
        return rsx! { div {} };
    }

    let heaviest = schedule
        .tasks
        .iter()
        .filter(|t| t.task.priority > 0)
        .max_by(|a, b| {
            a.task
                .emotional_weight
                .partial_cmp(&b.task.emotional_weight)
                .unwrap()
        });

    let suggestion = heaviest.map(|t| {
        format!(
            "Consider dropping '{}' (heaviest, priority {}).",
            t.task.name, t.task.priority
        )
    });

    rsx! {
        div { class: "overload-banner",
            div { class: "overload-icon", "⚠" }
            div { class: "overload-content",
                strong { "This schedule fits your time, but it might break you." }
                p { "Consider dropping or splitting a task to reduce cognitive load." }
                if let Some(s) = suggestion {
                    p { class: "overload-suggestion", "{s}" }
                }
            }
        }
    }
}
