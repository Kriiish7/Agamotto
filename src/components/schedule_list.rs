use crate::server::metrics;
use crate::state;
use dioxus::prelude::*;

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
pub fn ScheduleList() -> Element {
    let mut drag_from = use_signal(|| None::<usize>);

    let schedule = state::SCHEDULE.read();
    let schedule_ref = match schedule.as_ref() {
        Some(s) => s,
        None => return rsx! { div { "No schedule." } },
    };

    if schedule_ref.tasks.is_empty() {
        return rsx! {
            div { class: "schedule-list-empty",
                p { "No tasks fit within your time window." }
            }
        };
    }

    let tasks = schedule_ref.tasks.clone();

    rsx! {
        div { class: "schedule-list",
            h3 { "Scheduled Tasks" }
            for (i, st) in tasks.iter().enumerate() {
                div {
                    key: "{st.task.id}",
                    class: "schedule-row",
                    onmousedown: move |_| {
                        drag_from.set(Some(i));
                    },
                    onmouseup: move |_| {
                        if let Some(from) = drag_from() {
                            if from != i {
                                let all_tasks = state::TASKS.read().clone();
                                let mut s = state::SCHEDULE.write();
                                if let Some(ref mut sched) = *s {
                                    if from < sched.tasks.len() && i < sched.tasks.len() {
                                        let task = sched.tasks.remove(from);
                                        sched.tasks.insert(i, task);
                                        let mut time = 0;
                                        for t in sched.tasks.iter_mut() {
                                            t.start_time = time;
                                            time += t.task.duration;
                                            t.end_time = time;
                                        }
                                        metrics::compute_metrics(sched, &all_tasks);
                                    }
                                }
                            }
                            drag_from.set(None);
                        }
                    },
                    div { class: "schedule-row-handle", "⠿" }
                    div { class: "schedule-row-number", "{i + 1}." }
                    div { class: "schedule-row-info",
                        div { class: "schedule-row-name", "{st.task.name}" }
                        div { class: "schedule-row-meta",
                            span { "{st.task.duration} min" }
                            span { " • " }
                            span { "{format_time(st.start_time)}–{format_time(st.end_time)}" }
                            span { " • " }
                            span { "P{st.task.priority}" }
                        }
                    }
                    if st.deadline_risk > 0.5 {
                        div { class: "schedule-row-risk",
                            "⚠ {st.deadline_risk:.0}%"
                        }
                    }
                }
            }
        }
    }
}
