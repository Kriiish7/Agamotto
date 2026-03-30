use super::{
    excluded_list::ExcludedList, metrics_panel::MetricsPanel, saved_schedules::SavedSchedules,
    schedule_list::ScheduleList, timeline::Timeline,
};
use crate::state;
use dioxus::prelude::*;

#[component]
pub fn ScheduleView() -> Element {
    let schedule = state::SCHEDULE.read();

    if schedule.is_none() {
        return rsx! {
            div { class: "page schedule-page",
                div { class: "schedule-empty",
                    h2 { "No Schedule Yet" }
                    p { "Go to the " }
                    Link { to: crate::Route::Home {}, "Plan" }
                    p { " page to add tasks and generate a schedule." }
                }
            }
        };
    }

    let mode_label = match &schedule.as_ref().unwrap().mode {
        crate::types::ScheduleMode::Serenity => "Serenity Mode",
        crate::types::ScheduleMode::Crunch => "Crunch Mode",
    };

    rsx! {
        div { class: "page schedule-page",
            div { class: "schedule-header",
                div { class: "schedule-header-left",
                    h1 { "Your Schedule" }
                    span { class: "mode-badge", "{mode_label}" }
                }
                div { class: "schedule-header-right",
                    SavedSchedules {}
                    Link { to: crate::Route::Home {}, class: "btn btn-secondary",
                        "← Back to Plan"
                    }
                }
            }
            Timeline {}
            div { class: "schedule-body",
                div { class: "schedule-left",
                    ScheduleList {}
                    ExcludedList {}
                }
                div { class: "schedule-right",
                    MetricsPanel {}
                }
            }
        }
    }
}
