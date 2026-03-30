use super::{config_panel::ConfigPanel, task_form::TaskForm, task_list::TaskList};
use dioxus::prelude::*;

#[component]
pub fn Home() -> Element {
    rsx! {
        div {
            class: "page home-page",
            div { class: "home-left",
                TaskForm {}
                TaskList {}
            }
            div { class: "home-right",
                ConfigPanel {}
            }
        }
    }
}
