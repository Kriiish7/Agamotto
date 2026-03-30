use crate::state;
use dioxus::prelude::*;

fn priority_color(priority: u8) -> &'static str {
    match priority {
        1 | 2 => "#22c55e",
        3 => "#f59e0b",
        4 | 5 => "#ef4444",
        _ => "#94a3b8",
    }
}

fn format_time(minutes: u32) -> String {
    let h = minutes / 60;
    let m = minutes % 60;
    if h > 0 {
        format!("{h}:{m:02}")
    } else {
        format!("0:{m:02}")
    }
}

#[component]
pub fn Timeline() -> Element {
    let schedule = state::SCHEDULE.read();
    let schedule = match schedule.as_ref() {
        Some(s) => s,
        None => return rsx! { div { "No schedule to display." } },
    };

    let total_time = schedule.available_time as f64;

    rsx! {
        div { class: "timeline",
            h3 { "Timeline" }
            div { class: "timeline-bar",
                for st in &schedule.tasks {
                    div {
                        key: "{st.task.id}",
                        class: "timeline-block",
                        style: "width: {(st.task.duration as f64 / total_time * 100.0):.1}%; background-color: {priority_color(st.task.priority)};",
                        div { class: "timeline-label",
                            "{st.task.name}"
                        }
                        div { class: "timeline-tooltip",
                            strong { "{st.task.name}" }
                            div { "{format_time(st.start_time)} – {format_time(st.end_time)} ({st.task.duration} min)" }
                            div { "Priority: {st.task.priority}/5" }
                            if let Some(dl) = &st.task.deadline {
                                div { "Deadline: {dl}" }
                            }
                        }
                    }
                }
            }
            div { class: "timeline-axis",
                span { "0:00" }
                span { "{format_time(schedule.available_time)}" }
            }
        }
    }
}
