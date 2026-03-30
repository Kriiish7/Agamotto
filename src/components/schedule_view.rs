use dioxus::prelude::*;

#[component]
pub fn ScheduleView() -> Element {
    rsx! {
        div {
            class: "page schedule-page",
            h1 { "Your Schedule" }
            p { "Timeline, metrics, and reshuffling — coming in Phase 4." }
        }
    }
}
