use crate::server::simulation::{simulate_change, ScheduleChange};
use crate::state;
use dioxus::prelude::*;

#[component]
pub fn WhatIfPanel(task_id: String, task_name: String) -> Element {
    let mut showing = use_signal(|| false);
    let mut result = use_signal(|| None::<String>);

    let drop_id = task_id.clone();
    let delay_id = task_id.clone();

    rsx! {
        div { class: "what-if-panel",
            button {
                class: "btn btn-sm btn-secondary",
                onclick: move |_| showing.toggle(),
                if showing() { "Hide" } else { "What if?" }
            }
            if showing() {
                div { class: "what-if-options",
                    button {
                        class: "btn btn-sm btn-danger",
                        onclick: move |_| {
                            let tasks = state::TASKS.read().clone();
                            let schedule = match state::SCHEDULE.read().clone() {
                                Some(s) => s,
                                None => return,
                            };
                            let mode = schedule.mode.clone();
                            let w = schedule.available_time;
                            let sim = simulate_change(
                                &tasks, w, &mode, &ScheduleChange::RemoveTask(drop_id.clone()),
                            );
                            result.set(Some(sim.summary));
                        },
                        "Drop this"
                    }
                    button {
                        class: "btn btn-sm btn-secondary",
                        onclick: move |_| {
                            let tasks = state::TASKS.read().clone();
                            let schedule = match state::SCHEDULE.read().clone() {
                                Some(s) => s,
                                None => return,
                            };
                            let mode = schedule.mode.clone();
                            let w = schedule.available_time;
                            let sim = simulate_change(
                                &tasks, w, &mode, &ScheduleChange::DelayTask(delay_id.clone()),
                            );
                            result.set(Some(sim.summary));
                        },
                        "Delay this"
                    }
                    if let Some(summary) = result() {
                        div { class: "what-if-result", "{summary}" }
                    }
                }
            }
        }
    }
}
