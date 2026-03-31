use crate::state;
use dioxus::prelude::*;

#[component]
pub fn FailureMarkers() -> Element {
    let schedule = state::SCHEDULE.read();
    let schedule = match schedule.as_ref() {
        Some(s) => s,
        None => return rsx! { div {} },
    };

    if schedule.metrics.failure_points.is_empty() {
        return rsx! { div {} };
    }

    rsx! {
        div { class: "failure-markers",
            h3 { "Collapse Risk Forecast" }
            for fp in &schedule.metrics.failure_points {
                div { key: "{fp.task_id}", class: "failure-marker",
                    div { class: "failure-marker-icon", "⚠" }
                    div { class: "failure-marker-content",
                        div { class: "failure-marker-header",
                            strong { "High collapse risk" }
                            span { class: "failure-risk-score", "Risk: {fp.risk_score * 100.0:.0}%" }
                        }
                        p { class: "failure-marker-reason", "{fp.reason}" }
                    }
                }
            }
        }
    }
}
