use super::{
    config_panel::ConfigPanel, habit_alert::HabitAlert, task_form::TaskForm, task_list::TaskList,
};
use dioxus::prelude::*;

#[component]
pub fn Home() -> Element {
    rsx! {
        div {
            class: "page home-page",
            HabitAlert {}
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
