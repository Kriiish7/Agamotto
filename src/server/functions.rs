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
