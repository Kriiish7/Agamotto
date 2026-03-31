use crate::state;
use crate::types::Task;
use dioxus::prelude::*;

#[component]
pub fn TaskForm() -> Element {
    let mut name = use_signal(String::new);
    let mut duration = use_signal(|| 30u32);
    let mut priority = use_signal(|| 3u8);
    let mut deadline = use_signal(String::new);
    let mut tags = use_signal(String::new);
    let mut emotional_weight = use_signal(|| 0.0f64);
    let mut category = use_signal(String::new);

    let mut error_msg = use_signal(String::new);

    rsx! {
        div {
            class: "task-form",
            h2 { "Add Task" }

            if !error_msg().is_empty() {
                div { class: "form-error", "{error_msg}" }
            }

            div { class: "form-group",
                label { "Task Name" }
                input {
                    id: "task-name",
                    r#type: "text",
                    placeholder: "e.g. Revise circuits",
                    maxlength: "100",
                    value: name,
                    oninput: move |e| name.set(e.value()),
                }
            }

            div { class: "form-group",
                label { "Duration: {duration()} min" }
                input {
                    r#type: "range",
                    min: "5",
                    max: "480",
                    step: "5",
                    value: "{duration}",
                    oninput: move |e| {
                        if let Ok(v) = e.value().parse::<u32>() {
                            duration.set(v);
                        }
                    },
                }
                div { class: "range-labels",
                    span { "5 min" }
                    span { "4 hours" }
                    span { "8 hours" }
                }
            }

            div { class: "form-group",
                label { "Priority" }
                div { class: "priority-selector",
                    for i in 1..=5u8 {
                        button {
                            key: "{i}",
                            class: if i <= priority() { "star active" } else { "star" },
                            onclick: move |_| priority.set(i),
                            "★"
                        }
                    }
                    span { class: "priority-label",
                        {match priority() {
                            5 => "Critical",
                            4 => "High",
                            3 => "Medium",
                            2 => "Low",
                            1 => "Nice to do",
                            _ => "",
                        }}
                    }
                }
            }

            div { class: "form-group",
                label { "Deadline (optional)" }
                input {
                    id: "task-deadline",
                    r#type: "datetime-local",
                    value: deadline,
                    oninput: move |e| deadline.set(e.value()),
                }
            }

            div { class: "form-group",
                label { "Tags (comma-separated)" }
                input {
                    id: "task-tags",
                    r#type: "text",
                    placeholder: "e.g. revision, physics",
                    value: tags,
                    oninput: move |e| tags.set(e.value()),
                }
            }

            div { class: "form-group",
                label { "Category (optional)" }
                select {
                    value: category,
                    onchange: move |e| category.set(e.value()),
                    option { value: "", "None" }
                    option { value: "student", "Student" }
                    option { value: "developer", "Developer" }
                    option { value: "applicant", "Applicant" }
                    option { value: "personal", "Personal" }
                    option { value: "health", "Health" }
                }
            }

            div { class: "form-group",
                label { "Emotional Weight: {format_emotional(emotional_weight())}" }
                input {
                    r#type: "range",
                    min: "0",
                    max: "100",
                    step: "10",
                    value: "{(emotional_weight() * 100.0) as u32}",
                    oninput: move |e| {
                        if let Ok(v) = e.value().parse::<u32>() {
                            emotional_weight.set(v as f64 / 100.0);
                        }
                    },
                }
                div { class: "range-labels",
                    span { "Light" }
                    span { "Moderate" }
                    span { "Heavy" }
                }
            }

            button {
                class: "btn btn-primary",
                onclick: move |_| {
                    let n = name().trim().to_string();
                    if n.is_empty() {
                        error_msg.set("Task name is required".into());
                        return;
                    }
                    if duration() == 0 {
                        error_msg.set("Duration must be greater than 0".into());
                        return;
                    }

                    let parsed_tags: Vec<String> = tags()
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();

                    let dl = if deadline().is_empty() { None } else { Some(deadline()) };
                    let cat = if category().is_empty() { None } else { Some(category()) };

                    let task = Task {
                        id: crate::types::uuid_v4(),
                        name: n,
                        duration: duration(),
                        priority: priority(),
                        deadline: dl,
                        tags: parsed_tags,
                        emotional_weight: emotional_weight(),
                        category: cat,
                    };

                    state::TASKS.write().push(task);

                    // Reset form
                    name.set(String::new());
                    duration.set(30);
                    priority.set(3);
                    deadline.set(String::new());
                    tags.set(String::new());
                    category.set(String::new());
                    emotional_weight.set(0.0);
                    error_msg.set(String::new());
                },
                "Add Task"
            }
        }
    }
}

fn format_emotional(w: f64) -> String {
    if w < 0.33 {
        "Light".into()
    } else if w < 0.66 {
        "Moderate".into()
    } else {
        "Heavy".into()
    }
}
