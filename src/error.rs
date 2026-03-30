use std::fmt;

#[derive(Debug)]
pub enum ScheduleError {
    EmptyTaskList,
    InvalidDuration(String),
    InvalidPriority(String),
    InvalidDeadline(String),
    SchedulingFailed(String),
}

impl fmt::Display for ScheduleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyTaskList => write!(f, "No tasks provided"),
            Self::InvalidDuration(msg) => write!(f, "Invalid duration: {msg}"),
            Self::InvalidPriority(msg) => write!(f, "Invalid priority: {msg}"),
            Self::InvalidDeadline(msg) => write!(f, "Invalid deadline: {msg}"),
            Self::SchedulingFailed(msg) => write!(f, "Scheduling failed: {msg}"),
        }
    }
}

impl std::error::Error for ScheduleError {}
