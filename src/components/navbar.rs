use super::debt_counter::DebtCounter;
use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn Navbar() -> Element {
    rsx! {
        nav {
            class: "navbar",
            div {
                class: "navbar-brand",
                Link {
                    to: Route::Home {},
                    class: "navbar-logo",
                    "Agamotto"
                }
            }
            div {
                class: "navbar-links",
                Link {
                    to: Route::Home {},
                    class: "navbar-link",
                    "Plan"
                }
                Link {
                    to: Route::ScheduleView {},
                    class: "navbar-link",
                    "Schedule"
                }
                DebtCounter {}
            }
        }
    }
}
