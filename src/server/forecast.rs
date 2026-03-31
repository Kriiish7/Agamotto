use crate::types::{FailurePoint, ScheduledTask};

pub fn forecast_failure_points(tasks: &[ScheduledTask], available_time: u32) -> Vec<FailurePoint> {
    let n = tasks.len();
    if n == 0 {
        return Vec::new();
    }

    let total_duration: u32 = tasks.iter().map(|t| t.task.duration).sum();
    let cumulative_stress: f64 = tasks.iter().map(|t| t.stress_contribution).sum();

    tasks
        .iter()
        .enumerate()
        .map(|(i, st)| {
            let position_fatigue = i as f64 / (n - 1).max(1) as f64;
            let stress_at_start = if total_duration > 0 {
                let _elapsed: u32 = tasks[..i].iter().map(|t| t.task.duration).sum();
                let stress_so_far: f64 = tasks[..i].iter().map(|t| t.stress_contribution).sum();
                stress_so_far / available_time as f64
            } else {
                0.0
            };
            let emotional = st.task.emotional_weight;
            let deadline_risk = st.deadline_risk / 100.0;
            let duration_burden = if st.task.duration > 60 {
                st.task.duration as f64 / 120.0
            } else {
                0.0
            };
            let priority_energy_mismatch = (st.task.priority as f64 / 5.0) * position_fatigue;

            let risk_score = position_fatigue * 0.20
                + stress_at_start * 0.25
                + emotional * 0.20
                + deadline_risk * 0.15
                + duration_burden * 0.10
                + priority_energy_mismatch * 0.10;

            let mut reasons = Vec::new();
            if position_fatigue > 0.7 {
                reasons.push("late in the schedule");
            }
            if stress_at_start > 0.5 {
                reasons.push("high accumulated stress");
            }
            if emotional > 0.6 {
                reasons.push("psychologically heavy");
            }
            if deadline_risk > 0.5 {
                reasons.push("tight deadline");
            }
            if duration_burden > 0.5 {
                reasons.push("long duration");
            }
            if priority_energy_mismatch > 0.4 {
                reasons.push("high priority with low remaining energy");
            }

            let reason = if reasons.is_empty() {
                "Low risk".to_string()
            } else {
                format!("High collapse risk — {}", reasons.join(" + "))
            };

            FailurePoint {
                task_id: st.task.id.clone(),
                reason,
                risk_score: risk_score.min(1.0),
            }
        })
        .filter(|fp| fp.risk_score > 0.5)
        .collect()
}
