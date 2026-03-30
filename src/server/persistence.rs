use std::collections::BTreeMap;

use convex::Value;

use super::convex_client::AgamottoConvex;
use crate::types::{Schedule, ScheduleMetrics, ScheduleMode, ScheduledTask};

pub async fn save_schedule(
    name: &str,
    schedule: &Schedule,
    session_id: &str,
) -> Result<String, String> {
    let mut cx = AgamottoConvex::new().await?;

    let tasks_json =
        serde_json::to_string(&schedule.tasks).map_err(|e| format!("Serialise tasks: {e}"))?;
    let metrics_json =
        serde_json::to_string(&schedule.metrics).map_err(|e| format!("Serialise metrics: {e}"))?;

    let mode_str = match schedule.mode {
        ScheduleMode::Serenity => "serenity",
        ScheduleMode::Crunch => "crunch",
    };

    let args = maplit::btreemap! {
        "sessionId".into() => Value::String(session_id.into()),
        "name".into() => Value::String(name.into()),
        "mode".into() => Value::String(mode_str.into()),
        "available_time".into() => Value::Float64(schedule.available_time as f64),
        "tasks_json".into() => Value::String(tasks_json),
        "metrics_json".into() => Value::String(metrics_json),
    };

    let result = cx.mutation("schedules:save", args).await?;
    extract_id(&result)
}

pub async fn load_schedule(id: &str) -> Result<Schedule, String> {
    let mut cx = AgamottoConvex::new().await?;

    let args = maplit::btreemap! {
        "id".into() => Value::String(id.into()),
    };

    let result = cx.query("schedules:load", args).await?;
    doc_to_schedule(&result)
}

pub async fn list_schedules(session_id: &str) -> Result<Vec<ScheduleSummary>, String> {
    let mut cx = AgamottoConvex::new().await?;

    let args = maplit::btreemap! {
        "sessionId".into() => Value::String(session_id.into()),
    };

    let result = cx.query("schedules:list", args).await?;

    match result {
        Value::Array(items) => {
            let mut summaries = Vec::new();
            for item in items {
                if let Some(summary) = extract_summary(&item) {
                    summaries.push(summary);
                }
            }
            Ok(summaries)
        }
        _ => Ok(Vec::new()),
    }
}

pub async fn delete_schedule(id: &str) -> Result<(), String> {
    let mut cx = AgamottoConvex::new().await?;

    let args = maplit::btreemap! {
        "id".into() => Value::String(id.into()),
    };

    cx.mutation("schedules:remove", args).await?;
    Ok(())
}

#[derive(Clone, Debug)]
pub struct ScheduleSummary {
    pub id: String,
    pub name: String,
    pub mode: String,
    pub available_time: u32,
}

fn extract_id(value: &Value) -> Result<String, String> {
    match value {
        Value::String(s) => Ok(s.clone()),
        Value::Object(obj) => {
            if let Some(id) = obj.get("id") {
                Ok(format!("{id:?}"))
            } else {
                Ok(format!("{value:?}"))
            }
        }
        _ => Ok(format!("{value:?}")),
    }
}

fn extract_summary(value: &Value) -> Option<ScheduleSummary> {
    match value {
        Value::Object(obj) => {
            let id = match obj.get("id") {
                Some(Value::String(s)) => s.clone(),
                _ => return None,
            };
            let name = match obj.get("name") {
                Some(Value::String(s)) => s.clone(),
                _ => "Untitled".into(),
            };
            let mode = match obj.get("mode") {
                Some(Value::String(s)) => s.clone(),
                _ => "serenity".into(),
            };
            let available_time = match obj.get("available_time") {
                Some(Value::Float64(n)) => *n as u32,
                Some(Value::Int64(n)) => *n as u32,
                _ => 240,
            };
            Some(ScheduleSummary {
                id,
                name,
                mode,
                available_time,
            })
        }
        _ => None,
    }
}

fn doc_to_schedule(value: &Value) -> Result<Schedule, String> {
    match value {
        Value::Object(obj) => {
            let tasks_json = match obj.get("tasks_json") {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("Missing tasks_json".into()),
            };
            let metrics_json = match obj.get("metrics_json") {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("Missing metrics_json".into()),
            };
            let mode_str = match obj.get("mode") {
                Some(Value::String(s)) => s.as_str(),
                _ => "serenity",
            };
            let available_time = match obj.get("available_time") {
                Some(Value::Float64(n)) => *n as u32,
                Some(Value::Int64(n)) => *n as u32,
                _ => 240,
            };

            let tasks: Vec<ScheduledTask> =
                serde_json::from_str(&tasks_json).map_err(|e| format!("Deserialise tasks: {e}"))?;
            let metrics: ScheduleMetrics = serde_json::from_str(&metrics_json)
                .map_err(|e| format!("Deserialise metrics: {e}"))?;

            let mode = match mode_str {
                "crunch" => ScheduleMode::Crunch,
                _ => ScheduleMode::Serenity,
            };

            Ok(Schedule {
                mode,
                tasks,
                available_time,
                metrics,
            })
        }
        _ => Err("Unexpected response format".into()),
    }
}
