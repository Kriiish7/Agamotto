use crate::state;
use dioxus::prelude::*;

#[component]
pub fn IdentityAlerts() -> Element {
    let schedule = state::SCHEDULE.read();
    let schedule = match schedule.as_ref() {
        Some(s) => s,
        None => return rsx! { div {} },
    };

    if schedule.metrics.identity_conflicts.is_empty() {
        return rsx! { div {} };
    }

    rsx! {
        div { class: "identity-alerts",
            h3 { "Identity Conflicts" }
            for (i, conflict) in schedule.metrics.identity_conflicts.iter().enumerate() {
                div { key: "{i}", class: "identity-alert",
                    div { class: "identity-alert-icon", "🎭" }
                    div { class: "identity-alert-content",
                        p { "{conflict}" }
                        p { class: "identity-alert-suggestion",
                            "Consider splitting these across different days."
                        }
                    }
                }
            }
        }
    }
}
