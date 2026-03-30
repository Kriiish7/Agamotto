use crate::server::functions;
use crate::state;
use dioxus::prelude::*;

#[component]
pub fn SavedSchedules() -> Element {
    let mut save_name = use_signal(String::new);
    let mut saving = use_signal(|| false);
    let mut status = use_signal(String::new);

    rsx! {
        div { class: "saved-schedules",
            div { class: "save-form",
                input {
                    r#type: "text",
                    placeholder: "Schedule name...",
                    value: save_name,
                    oninput: move |e| save_name.set(e.value()),
                }
                button {
                    class: "btn btn-primary btn-sm",
                    disabled: saving() || state::SCHEDULE.read().is_none(),
                    onclick: move |_| {
                        let name = if save_name().trim().is_empty() {
                            "Untitled Schedule".to_string()
                        } else {
                            save_name().trim().to_string()
                        };
                        let schedule = match state::SCHEDULE.read().clone() {
                            Some(s) => s,
                            None => return,
                        };
                        let session = state::session_id();
                        saving.set(true);
                        status.set("Saving...".into());

                        spawn(async move {
                            match functions::save_schedule(name, schedule, session).await {
                                Ok(_id) => {
                                    status.set("Saved!".into());
                                    save_name.set(String::new());
                                }
                                Err(e) => {
                                    status.set(format!("Error: {e}"));
                                }
                            }
                            saving.set(false);
                        });
                    },
                    if saving() { "Saving..." } else { "Save" }
                }
            }
            if !status().is_empty() {
                div { class: "save-status", "{status}" }
            }
        }
    }
}
