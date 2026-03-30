use std::collections::BTreeMap;

use convex::Value;

use super::convex_client::AgamottoConvex;
use crate::types::Schedule;

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
        crate::types::ScheduleMode::Serenity => "serenity",
        crate::types::ScheduleMode::Crunch => "crunch",
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
    Ok(format!("{result:?}"))
}

pub async fn load_schedule(id: &str) -> Result<Schedule, String> {
    let mut cx = AgamottoConvex::new().await?;

    let args = maplit::btreemap! {
        "id".into() => Value::String(id.into()),
    };

    let _result = cx.query("schedules:load", args).await?;

    Err("Schedule deserialisation not yet implemented".into())
}

pub async fn list_schedules(session_id: &str) -> Result<Vec<(String, String)>, String> {
    let mut cx = AgamottoConvex::new().await?;

    let args = maplit::btreemap! {
        "sessionId".into() => Value::String(session_id.into()),
    };

    let _result = cx.query("schedules:list", args).await?;

    Ok(Vec::new())
}
