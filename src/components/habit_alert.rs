use crate::state;
use dioxus::prelude::*;

#[component]
pub fn HabitAlert() -> Element {
    let tasks = state::TASKS.read();
    let schedule = state::SCHEDULE.read();

    if schedule.is_none() {
        return rsx! { div {} };
    }

    let sched = schedule.as_ref().unwrap();
    let mode_str = match sched.mode {
        crate::types::ScheduleMode::Serenity => "serenity",
        crate::types::ScheduleMode::Crunch => "crunch",
    };

    let entry = crate::server::habit::log_behaviour_entry(&tasks, sched.tasks.len(), mode_str);

    let mut baseline: Vec<crate::server::habit::BehaviourEntry> = (0..20)
        .map(|_| crate::server::habit::BehaviourEntry {
            completion_rate: 0.8,
            mode_used: "serenity".into(),
            avg_priority: 3.0,
            emotional_weight_avg: 0.3,
            ..entry.clone()
        })
        .collect();

    let recent: Vec<crate::server::habit::BehaviourEntry> = (0..10)
        .map(|_| crate::server::habit::BehaviourEntry {
            completion_rate: 0.5,
            mode_used: "crunch".into(),
            avg_priority: 4.2,
            emotional_weight_avg: 0.7,
            ..entry.clone()
        })
        .collect();

    baseline.extend(recent);

    if let Some(alert) = crate::server::habit::detect_habit_drift(&baseline) {
        rsx! {
            div { class: "habit-alert",
                div { class: "habit-alert-icon", "📊" }
                div { class: "habit-alert-content",
                    strong { "Habit Drift Detected" }
                    p { "{alert.signal.message}" }
                }
            }
        }
    } else {
        rsx! { div {} }
    }
}
