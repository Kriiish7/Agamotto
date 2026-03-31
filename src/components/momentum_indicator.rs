use crate::state;
use dioxus::prelude::*;

#[component]
pub fn MomentumIndicator() -> Element {
    let schedule = state::SCHEDULE.read();
    let schedule = match schedule.as_ref() {
        Some(s) => s,
        None => return rsx! { div {} },
    };

    let score = schedule.metrics.momentum_score;
    let (color, label) = if score >= 0.6 {
        ("#22c55e", "Good momentum — easy start, gradual ramp")
    } else if score >= 0.3 {
        ("#f59e0b", "Moderate momentum")
    } else {
        ("#ef4444", "Bad momentum — heavy task first, no warmup")
    };

    rsx! {
        div { class: "momentum-indicator",
            div { class: "metric-label", "Momentum" }
            div { class: "metric-value", "{score:.2}" }
            div { class: "metric-bar-track",
                div {
                    class: "metric-bar-fill",
                    style: "width: {(score * 100.0).min(100.0)}%; background-color: {color};"
                }
            }
            div { class: "metric-tooltip", "{label}" }
        }
    }
}
