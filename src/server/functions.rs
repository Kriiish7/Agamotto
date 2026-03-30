use dioxus::prelude::*;
use crate::types::{Schedule, Task};

#[server]
pub async fn schedule_serenity(tasks: Vec<Task>, available_time: u32) -> Result<Schedule, ServerFnError> {
    Ok(crate::server::greedy::schedule_serenity(&tasks, available_time))
}

#[server]
pub async fn schedule_crunch(tasks: Vec<Task>, available_time: u32) -> Result<Schedule, ServerFnError> {
    Ok(crate::server::knapsack::schedule_crunch(&tasks, available_time))
}

#[server]
pub async fn save_schedule(name: String, schedule: Schedule, session_id: String) -> Result<String, ServerFnError> {
    crate::server::persistence::save_schedule(&name, &schedule, &session_id)
        .await
        .map_err(|e| ServerFnError::new(e))
}

#[server]
pub async fn load_schedule(id: String) -> Result<Schedule, ServerFnError> {
    crate::server::persistence::load_schedule(&id)
        .await
        .map_err(|e| ServerFnError::new(e))
}

#[server]
pub async fn list_schedules(session_id: String) -> Result<Vec<ScheduleSummaryDto>, ServerFnError> {
    let summaries = crate::server::persistence::list_schedules(&session_id)
        .await
        .map_err(|e| ServerFnError::new(e))?;
    Ok(summaries
        .into_iter()
        .map(|s| ScheduleSummaryDto {
            id: s.id,
            name: s.name,
            mode: s.mode,
            available_time: s.available_time,
        })
        .collect())
}

#[server]
pub async fn delete_schedule(id: String) -> Result<(), ServerFnError> {
    crate::server::persistence::delete_schedule(&id)
        .await
        .map_err(|e| ServerFnError::new(e))
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ScheduleSummaryDto {
    pub id: String,
    pub name: String,
    pub mode: String,
    pub available_time: u32,
}
