use crate::server::functions;
use crate::state;
use crate::types::ScheduleMode;
use dioxus::prelude::*;

#[component]
pub fn ConfigPanel() -> Element {
    let mut hours = use_signal(|| 4u32);
    let mut minutes = use_signal(|| 0u32);
    let mut generating = use_signal(|| false);
    let mut error_msg = use_signal(String::new);

    let total_minutes = move || hours() * 60 + minutes();

    rsx! {
        div { class: "config-panel",
            h2 { "Configuration" }

            if !error_msg().is_empty() {
                div { class: "form-error", "{error_msg}" }
            }

            div { class: "form-group",
                label { "Available Time" }
                div { class: "time-input",
                    div { class: "time-field",
                        input {
                            r#type: "number",
                            min: "0",
                            max: "12",
                            value: "{hours}",
                            oninput: move |e| {
                                if let Ok(v) = e.value().parse::<u32>() {
                                    hours.set(v.min(12));
                                }
                            },
                        }
                        span { "hrs" }
                    }
                    div { class: "time-field",
                        input {
                            r#type: "number",
                            min: "0",
                            max: "59",
                            step: "15",
                            value: "{minutes}",
                            oninput: move |e| {
                                if let Ok(v) = e.value().parse::<u32>() {
                                    minutes.set(v.min(59));
                                }
                            },
                        }
                        span { "min" }
                    }
                }
                div { class: "time-total", "Total: {total_minutes()} minutes" }
            }

            div { class: "form-group",
                label { "Scheduling Mode" }
                div { class: "mode-toggle",
                    button {
                        class: if *state::MODE.read() == ScheduleMode::Serenity { "mode-btn active" } else { "mode-btn" },
                        onclick: move |_| { *state::MODE.write() = ScheduleMode::Serenity; },
                        div { class: "mode-name", "Serenity" }
                        div { class: "mode-desc", "Relaxed — quick wins first" }
                    }
                    button {
                        class: if *state::MODE.read() == ScheduleMode::Crunch { "mode-btn active" } else { "mode-btn" },
                        onclick: move |_| { *state::MODE.write() = ScheduleMode::Crunch; },
                        div { class: "mode-name", "Crunch" }
                        div { class: "mode-desc", "Deadline-driven — optimal selection" }
                    }
                }
            }

            button {
                class: "btn btn-primary btn-generate",
                disabled: state::TASKS.read().is_empty() || generating(),
                onclick: move |_| {
                    let tasks = state::TASKS.read().clone();
                    let w = total_minutes();
                    let mode = state::MODE.read().clone();
                    generating.set(true);
                    error_msg.set(String::new());

                    spawn(async move {
                        let result = match mode {
                            ScheduleMode::Serenity => {
                                functions::schedule_serenity(tasks, w).await
                            }
                            ScheduleMode::Crunch => {
                                functions::schedule_crunch(tasks, w).await
                            }
                        };

                        match result {
                            Ok(schedule) => {
                                *state::SCHEDULE.write() = Some(schedule);
                                let nav = navigator();
                                nav.push(crate::Route::ScheduleView {});
                            }
                            Err(e) => {
                                error_msg.set(format!("Error: {e}"));
                            }
                        }
                        generating.set(false);
                    });
                },
                if generating() {
                    "Generating..."
                } else if state::TASKS.read().is_empty() {
                    "Add tasks to generate"
                } else {
                    "Generate Schedule"
                }
            }
        }
    }
}
