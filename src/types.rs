use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub duration: u32,
    pub priority: u8,
    pub deadline: Option<String>,
    pub tags: Vec<String>,
    pub emotional_weight: f64,
    pub category: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ScheduleMode {
    Serenity,
    Crunch,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Schedule {
    pub mode: ScheduleMode,
    pub tasks: Vec<ScheduledTask>,
    pub available_time: u32,
    pub metrics: ScheduleMetrics,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub task: Task,
    pub start_time: u32,
    pub end_time: u32,
    pub stress_contribution: f64,
    pub deadline_risk: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScheduleMetrics {
    pub productivity_score: f64,
    pub time_utilisation: f64,
    pub stress_index: f64,
    pub deadline_risk: f64,
    pub overload_flag: bool,
    pub failure_points: Vec<FailurePoint>,
    pub decision_debt: u32,
    pub identity_conflicts: Vec<String>,
    pub momentum_score: f64,
    pub habit_drift_alert: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FailurePoint {
    pub task_id: String,
    pub reason: String,
    pub risk_score: f64,
}

impl Task {
    pub fn new(name: impl Into<String>, duration: u32, priority: u8) -> Self {
        Self {
            id: uuid_v4(),
            name: name.into(),
            duration,
            priority,
            deadline: None,
            tags: Vec::new(),
            emotional_weight: 0.0,
            category: None,
        }
    }
}

impl Default for ScheduleMetrics {
    fn default() -> Self {
        Self {
            productivity_score: 0.0,
            time_utilisation: 0.0,
            stress_index: 0.0,
            deadline_risk: 0.0,
            overload_flag: false,
            failure_points: Vec::new(),
            decision_debt: 0,
            identity_conflicts: Vec::new(),
            momentum_score: 0.0,
            habit_drift_alert: false,
        }
    }
}

fn uuid_v4() -> String {
    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        rand_u32(),
        rand_u16(),
        rand_u16(),
        rand_u16(),
        rand_u64() & 0x0000_FFFF_FFFF_FFFF
    )
}

fn rand_u32() -> u32 {
    static mut SEED: u64 = 0xdead_beef_cafe_babe;
    unsafe {
        SEED ^= SEED << 13;
        SEED ^= SEED >> 7;
        SEED ^= SEED << 17;
        SEED as u32
    }
}

fn rand_u16() -> u16 {
    rand_u32() as u16
}

fn rand_u64() -> u64 {
    ((rand_u32() as u64) << 32) | (rand_u32() as u64)
}
