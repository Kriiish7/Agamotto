use crate::state;
use dioxus::prelude::*;

#[component]
pub fn DebtCounter() -> Element {
    let tasks = state::TASKS.read();
    let debt = crate::server::debt::compute_decision_debt(&tasks);

    if debt == 0 {
        return rsx! { div {} };
    }

    let (color, label) = if debt <= 3 {
        ("#22c55e", "Healthy")
    } else if debt <= 7 {
        ("#f59e0b", "Some open decisions")
    } else if debt <= 10 {
        ("#f97316", "Clear some choices")
    } else {
        ("#ef4444", "Decision debt is critical")
    };

    rsx! {
        div { class: "debt-counter",
            span {
                class: "debt-badge",
                style: "background-color: {color};",
                "{debt}"
            }
            span { class: "debt-label", "open decisions" }
        }
    }
}
