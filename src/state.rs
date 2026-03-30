use crate::types::{Schedule, ScheduleMode, Task};
use dioxus::prelude::*;

pub static TASKS: GlobalSignal<Vec<Task>> = Signal::global(Vec::new);
pub static SCHEDULE: GlobalSignal<Option<Schedule>> = Signal::global(|| None);
pub static AVAILABLE_TIME: GlobalSignal<u32> = Signal::global(|| 240);
pub static MODE: GlobalSignal<ScheduleMode> = Signal::global(|| ScheduleMode::Serenity);
