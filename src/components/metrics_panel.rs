use crate::state;
use dioxus::prelude::*;

fn bar_color(value: f64, thresholds: (f64, f64)) -> &'static str {
    if value < thresholds.0 {
        "#22c55e"
    } else if value < thresholds.1 {
        "#f59e0b"
    } else {
        "#ef4444"
    }
}

#[component]
pub fn MetricsPanel() -> Element {
    let schedule = state::SCHEDULE.read();
    let metrics = match schedule.as_ref() {
        Some(s) => &s.metrics,
        None => return rsx! { div { "No metrics to display." } },
    };

    rsx! {
        div { class: "metrics-panel",
            h3 { "Analytics" }
            div { class: "metrics-grid",
                div { class: "metric-card",
                    div { class: "metric-label", "Productivity Score" }
                    div { class: "metric-value", "{metrics.productivity_score:.0}%" }
                    div { class: "metric-bar-track",
                        div {
                            class: "metric-bar-fill",
                            style: "width: {metrics.productivity_score.min(100.0)}%; background-color: #3b82f6;"
                        }
                    }
                    div { class: "metric-tooltip", "Percentage of total priority value captured" }
                }

                div { class: "metric-card",
                    div { class: "metric-label", "Time Utilisation" }
                    div { class: "metric-value", "{metrics.time_utilisation:.0}%" }
                    div { class: "metric-bar-track",
                        div {
                            class: "metric-bar-fill",
                            style: "width: {metrics.time_utilisation.min(100.0)}%; background-color: #6366f1;"
                        }
                    }
                    div { class: "metric-tooltip", "Percentage of available time used" }
                }

                div { class: "metric-card",
                    div { class: "metric-label", "Stress Index" }
                    div { class: "metric-value", "{metrics.stress_index:.2}" }
                    div { class: "metric-bar-track",
                        div {
                            class: "metric-bar-fill",
                            style: "width: {(metrics.stress_index * 100.0).min(100.0)}%; background-color: {bar_color(metrics.stress_index, (0.4, 0.7))};"
                        }
                    }
                    div { class: "metric-tooltip",
                        if metrics.stress_index < 0.4 {
                            "Comfortable schedule"
                        } else if metrics.stress_index < 0.7 {
                            "Moderate pressure"
                        } else {
                            "High pressure — consider easing up"
                        }
                    }
                }

                div { class: "metric-card",
                    div { class: "metric-label", "Deadline Risk" }
                    div { class: "metric-value", "{metrics.deadline_risk:.0}%" }
                    div { class: "metric-bar-track",
                        div {
                            class: "metric-bar-fill",
                            style: "width: {metrics.deadline_risk.min(100.0)}%; background-color: {bar_color(metrics.deadline_risk / 100.0, (0.3, 0.6))};"
                        }
                    }
                    div { class: "metric-tooltip", "Average probability of missing deadlines" }
                }
            }
        }
    }
}
