use dioxus::prelude::*;

#[component]
pub fn Home() -> Element {
    rsx! {
        div {
            class: "page home-page",
            h1 { "Plan Your Day" }
            p { "Task input, configuration, and schedule generation — coming in Phase 3." }
        }
    }
}
