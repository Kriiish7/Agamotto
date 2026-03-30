use crate::state;
use dioxus::prelude::*;

#[component]
pub fn TaskList() -> Element {
    let tasks = state::TASKS.read();

    if tasks.is_empty() {
        return rsx! {
            div { class: "task-list-empty",
                p { "No tasks yet. Add your first task above." }
            }
        };
    }

    rsx! {
        div { class: "task-list",
            h3 { "Your Tasks ({tasks.len()})" }
            for (i, task) in tasks.iter().enumerate() {
                div { key: "{task.id}", class: "task-row",
                    div { class: "task-row-main",
                        span { class: "task-number", "{i + 1}." }
                        span { class: "task-name", "{task.name}" }
                        span { class: "task-duration", "{task.duration} min" }
                        span { class: "task-priority",
                            {(0..task.priority).map(|_| "★").collect::<String>()}
                        }
                    }
                    if let Some(dl) = &task.deadline {
                        div { class: "task-row-meta",
                            span { class: "task-deadline", "Due: {dl}" }
                        }
                    }
                    if !task.tags.is_empty() {
                        div { class: "task-row-tags",
                            for tag in &task.tags {
                                span { key: "{tag}", class: "tag", "{tag}" }
                            }
                        }
                    }
                    div { class: "task-row-actions",
                        button {
                            class: "btn btn-sm btn-danger",
                            onclick: move |_| {
                                state::TASKS.write().remove(i);
                            },
                            "Delete"
                        }
                    }
                }
            }
        }
    }
}
