# Phase 8: Detection Algorithms & Specifications

> Detailed algorithmic specifications for all 10 future-ready features.
> This document defines *exactly* how each detection mechanism works.

---

## 1. Habit Drift Detection

**Problem:** The user's scheduling behaviour changes over time (e.g. they used to plan calmly, now they're always in Crunch Mode) but they don't notice until they're burned out.

**Data source:** `behaviour_log` table in Convex. One entry per schedule generation.

### What gets logged

```rust
struct BehaviourEntry {
    session_id: String,
    timestamp: String,           // ISO 8601
    tasks_planned: u32,          // how many tasks user entered
    tasks_scheduled: u32,        // how many made it into the schedule
    completion_rate: f64,        // user-reported: tasks_completed / tasks_scheduled
    avg_priority_selected: f64,  // mean priority of tasks that made it into schedule
    mode_used: ScheduleMode,     // Serenity or Crunch
    avg_emotional_weight: f64,   // mean emotional_weight of scheduled tasks
    available_time: u32,         // W for that session
}
```

### Detection algorithm

```
fn detect_habit_drift(entries: &[BehaviourEntry]) -> Option<DriftAlert> {
    if entries.len() < 14 { return None }  // need at least 2 weeks of data

    let recent = &entries[entries.len()-7..];   // last 7 sessions
    let baseline = &entries[..entries.len()-7]; // everything before that

    let mut alerts = Vec::new();

    // DRIFT 1: Mode shift
    // Count how often Crunch was used in each window
    let baseline_crunch_rate = baseline.iter()
        .filter(|e| e.mode_used == Crunch).count() as f64 / baseline.len() as f64;
    let recent_crunch_rate = recent.iter()
        .filter(|e| e.mode_used == Crunch).count() as f64 / recent.len() as f64;

    if recent_crunch_rate - baseline_crunch_rate > 0.4 {
        // Was 20% Crunch, now 70% — big shift
        alerts.push(DriftSignal {
            kind: DriftKind::ModeShift,
            magnitude: recent_crunch_rate - baseline_crunch_rate,
            message: format!(
                "You've switched to Crunch Mode {}% of the time recently (was {}% before). \
                 Are you under more pressure?",
                (recent_crunch_rate * 100.0) as u32,
                (baseline_crunch_rate * 100.0) as u32
            ),
        });
    }

    // DRIFT 2: Priority creep
    // Rising average priority = increasingly urgent tasks dominating
    let baseline_avg_pri = mean(baseline, |e| e.avg_priority_selected);
    let recent_avg_pri = mean(recent, |e| e.avg_priority_selected);

    if recent_avg_pri - baseline_avg_pri > 0.8 {
        alerts.push(DriftSignal {
            kind: DriftKind::PriorityCreep,
            magnitude: recent_avg_pri - baseline_avg_pri,
            message: format!(
                "Your average task priority has risen from {:.1} to {:.1}. \
                 Everything feels urgent now — that's a warning sign.",
                baseline_avg_pri, recent_avg_pri
            ),
        });
    }

    // DRIFT 3: Completion rate decline
    let baseline_completion = mean(baseline, |e| e.completion_rate);
    let recent_completion = mean(recent, |e| e.completion_rate);

    if baseline_completion - recent_completion > 0.15 {
        // Was completing 80%, now 60%
        alerts.push(DriftSignal {
            kind: DriftKind::CompletionDecline,
            magnitude: baseline_completion - recent_completion,
            message: format!(
                "Your completion rate dropped from {}% to {}%. \
                 You might be overloading yourself.",
                (baseline_completion * 100.0) as u32,
                (recent_completion * 100.0) as u32
            ),
        });
    }

    // DRIFT 4: Emotional weight trend
    let baseline_emotional = mean(baseline, |e| e.avg_emotional_weight);
    let recent_emotional = mean(recent, |e| e.avg_emotional_weight);

    if recent_emotional - baseline_emotional > 0.2 {
        alerts.push(DriftSignal {
            kind: DriftKind::EmotionalHeaviness,
            magnitude: recent_emotional - baseline_emotional,
            message: format!(
                "Tasks are feeling heavier lately (emotional weight: {:.2} → {:.2}). \
                 Consider whether you need a lighter week.",
                baseline_emotional, recent_emotional
            ),
        });
    }

    // DRIFT 5: Overplanning
    let baseline_planned = mean(baseline, |e| e.tasks_planned as f64);
    let recent_planned = mean(recent, |e| e.tasks_planned as f64);

    if recent_planned / baseline_planned > 1.5 {
        alerts.push(DriftSignal {
            kind: DriftKind::Overplanning,
            magnitude: recent_planned / baseline_planned,
            message: format!(
                "You're entering {:.0} tasks per session now (was {:.0}). \
                 More tasks ≠ more productivity.",
                recent_planned, baseline_planned
            ),
        });
    }

    // DRIFT 6: Time pressure escalation
    let baseline_time = mean(baseline, |e| e.available_time as f64);
    let recent_time = mean(recent, |e| e.available_time as f64);

    if baseline_time - recent_time > 60.0 {
        // Available time shrinking while task count grows
        alerts.push(DriftSignal {
            kind: DriftKind::TimeShrink,
            magnitude: (baseline_time - recent_time) / baseline_time,
            message: format!(
                "Your available time window has shrunk from {:.0}min to {:.0}min. \
                 Are you giving yourself less room to breathe?",
                baseline_time, recent_time
            ),
        });
    }

    // Return highest-magnitude alert, or None if no drift
    alerts.into_iter()
        .max_by(|a, b| a.magnitude.partial_cmp(&b.magnitude).unwrap())
        .map(|signal| DriftAlert { signal, detected_at: now() })
}
```

### Thresholds (tunable)

| Drift | Threshold | Rationale |
|-------|-----------|-----------|
| Mode shift | Δ > 0.4 in Crunch rate | 40% swing = significant behaviour change |
| Priority creep | Δ > 0.8 in avg priority | Moving from "medium" to "high" on average |
| Completion decline | Δ > 0.15 in completion rate | 15% drop = consistently failing to finish |
| Emotional heaviness | Δ > 0.2 in avg emotional weight | Perceptible shift in how tasks feel |
| Overplanning | Ratio > 1.5x tasks per session | 50% more tasks = scope creep |
| Time shrink | Δ > 60 min in available window | Losing an hour of planning space |

### When it runs

- `check_habit_drift()` is called on Home page load (once per session)
- Only triggers if user has ≥ 14 behaviour log entries
- Shows at most 1 alert per session (highest magnitude wins)
- User can dismiss; dismissal is not logged (no nagging)

---

## 2. Invisible Overload Detection

**Problem:** A schedule fits within W, passes all constraints, and looks "productive" — but it will feel crushing to actually execute.

### Detection signals

```rust
fn detect_overload(schedule: &Schedule) -> bool {
    let n = schedule.tasks.len();
    let total_duration: u32 = schedule.tasks.iter().map(|t| t.task.duration).sum();
    let utilisation = total_duration as f64 / schedule.available_time as f64;

    // SIGNAL 1: High utilisation + high stress
    // Filling 95% of your time with high-stress tasks = recipe for burnout
    let overload_utilisation_stress =
        utilisation > 0.90 && schedule.metrics.stress_index > 0.65;

    // SIGNAL 2: High average emotional weight
    // Even if individual tasks are short, if they ALL feel heavy...
    let avg_emotional: f64 = schedule.tasks.iter()
        .map(|t| t.task.emotional_weight)
        .sum::<f64>() / n as f64;
    let overload_emotional = avg_emotional > 0.5;

    // SIGNAL 3: Back-to-back high-emotional-weight tasks
    // Three heavy tasks in a row with no break = guaranteed fatigue
    let mut consecutive_heavy = 0;
    let mut max_consecutive = 0;
    for scheduled in &schedule.tasks {
        if scheduled.task.emotional_weight > 0.6 && scheduled.task.priority >= 4 {
            consecutive_heavy += 1;
            max_consecutive = max_consecutive.max(consecutive_heavy);
        } else {
            consecutive_heavy = 0;
        }
    }
    let overload_consecutive = max_consecutive >= 3;

    // SIGNAL 4: Long tasks at the end
    // A 90-minute task scheduled last, after 3 hours of work = will be done badly
    let last_third = &schedule.tasks[(n * 2 / 3)..];
    let tail_heavy = last_third.iter().any(|t| {
        t.task.duration > 60 && t.task.emotional_weight > 0.5
    });

    // COMBINE: overload if any 2+ signals fire
    let signals = [
        overload_utilisation_stress,
        overload_emotional,
        overload_consecutive,
        tail_heavy,
    ];
    signals.iter().filter(|&&s| s).count() >= 2
}
```

### UI response

- Warning banner: "This schedule fits your time, but it might break you."
- Specific suggestion: "Consider dropping [highest emotional weight, lowest priority task] or inserting a break after [first consecutive-heavy cluster]."
- Dismissable — user can acknowledge and proceed

---

## 3. Failure Forecasting

**Problem:** Some tasks in the schedule are more likely to be done poorly, skipped, or rushed. Predict which ones *before* the user gets there.

### Scoring each scheduled task

```rust
fn forecast_failure(schedule: &Schedule) -> Vec<FailurePoint> {
    let n = schedule.tasks.len();
    let total_duration: u32 = schedule.tasks.iter().map(|t| t.task.duration).sum();

    schedule.tasks.iter().enumerate().map(|(i, scheduled)| {
        // FACTOR 1: Position fatigue (later = worse)
        // Linear ramp from 0.0 (first task) to 1.0 (last task)
        let position_fatigue = i as f64 / (n - 1).max(1) as f64;

        // FACTOR 2: Cumulative stress at this point
        // How stressed is the user when they START this task?
        let cumulative_stress: f64 = schedule.tasks[..i].iter()
            .map(|t| t.stress_contribution)
            .sum::<f64>();
        let stress_at_start = cumulative_stress
            / schedule.available_time as f64;  // normalise

        // FACTOR 3: Emotional weight of THIS task
        let emotional = scheduled.task.emotional_weight;

        // FACTOR 4: Deadline proximity risk
        // How close is the deadline when this task is scheduled to START?
        let deadline_risk = if let Some(deadline) = &scheduled.task.deadline {
            let hours_until = hours_between(now(), parse_iso(deadline));
            let hours_at_start = hours_until
                - (scheduled.start_time as f64 / 60.0);
            // Risk increases as deadline approaches
            if hours_at_start <= 0.0 { 1.0 }       // already past deadline
            else if hours_at_start < 2.0 { 0.9 }   // less than 2 hours
            else if hours_at_start < 6.0 { 0.6 }   // less than 6 hours
            else if hours_at_start < 24.0 { 0.3 }  // less than a day
            else { 0.1 }
        } else {
            0.0  // no deadline = no deadline risk
        };

        // FACTOR 5: Duration burden
        // Long tasks late in the schedule are harder to sustain focus for
        let duration_burden = if scheduled.task.duration > 60 {
            scheduled.task.duration as f64 / 120.0  // 90min task = 0.75
        } else {
            0.0
        };

        // FACTOR 6: Priority mismatch
        // High-priority task done with low remaining energy = risky
        let priority_energy_mismatch =
            (scheduled.task.priority as f64 / 5.0) * position_fatigue;

        // WEIGHTED COMBINATION
        let risk_score =
            position_fatigue       * 0.20 +  // being late in the day
            stress_at_start        * 0.25 +  // accumulated stress
            emotional              * 0.20 +  // task feels heavy
            deadline_risk          * 0.15 +  // deadline pressure
            duration_burden        * 0.10 +  // task is long
            priority_energy_mismatch * 0.10;  // energy/priority mismatch

        let reason = build_failure_reason(
            position_fatigue, stress_at_start, emotional,
            deadline_risk, duration_burden, priority_energy_mismatch
        );

        FailurePoint {
            task_id: scheduled.task.id.clone(),
            task_name: scheduled.task.name.clone(),
            risk_score: risk_score.min(1.0),
            reason,
            position: i,
        }
    })
    .filter(|fp| fp.risk_score > 0.5)  // only flag high-risk tasks
    .sorted_by(|a, b| b.risk_score.partial_cmp(&a.risk_score).unwrap())
    .collect()
}

fn build_failure_reason(
    position: f64, stress: f64, emotional: f64,
    deadline: f64, duration: f64, mismatch: f64,
) -> String {
    let mut reasons = Vec::new();
    if position > 0.7 { reasons.push("late in the schedule"); }
    if stress > 0.5 { reasons.push("high accumulated stress"); }
    if emotional > 0.6 { reasons.push("psychologically heavy task"); }
    if deadline > 0.5 { reasons.push("tight deadline"); }
    if duration > 0.5 { reasons.push("long duration"); }
    if mismatch > 0.4 { reasons.push("high-priority task with low remaining energy"); }

    format!("High collapse risk — {}", reasons.join(" + "))
}
```

### UI

- Red warning icons on the timeline at each failure point position
- Hover: "⚠ High collapse risk — late in the schedule + high accumulated stress + psychologically heavy task"
- Analytics panel: ranked list of failure points with risk scores

---

## 4. Identity Conflict Alerts

**Problem:** A student trying to be a university applicant, a coursework student, a developer, and a person with a social life — all in one schedule — is set up to fail at all of them.

### Detection algorithm

```rust
fn detect_identity_conflicts(tasks: &[ScheduledTask]) -> Vec<IdentityConflict> {
    // Group tasks by category
    let mut categories: HashMap<String, Vec<&ScheduledTask>> = HashMap::new();
    for t in tasks {
        let cat = t.task.category.clone().unwrap_or_else(|| "uncategorised".into());
        categories.entry(cat).or_default().push(t);
    }

    let distinct_categories: Vec<&String> = categories.keys().collect();
    let mut conflicts = Vec::new();

    // CONFLICT 1: Too many hats
    if distinct_categories.len() >= 4 {
        conflicts.push(IdentityConflict {
            severity: 0.8,
            message: format!(
                "You're wearing {} hats today: {}. \
                 Consider focusing on 2-3 roles max.",
                distinct_categories.len(),
                distinct_categories.iter()
                    .map(|c| c.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            suggestion: "Split these categories across different days.".into(),
        });
    }

    // CONFLICT 2: Competing high-priority identities
    // Two different categories both have high-priority tasks
    let high_pri_by_cat: HashMap<&str, Vec<&ScheduledTask>> = tasks.iter()
        .filter(|t| t.task.priority >= 4)
        .filter_map(|t| t.task.category.as_deref().map(|cat| (cat, t)))
        .fold(HashMap::new(), |mut acc, (cat, t)| {
            acc.entry(cat).or_insert_with(Vec::new).push(t);
            acc
        });

    let high_pri_cats: Vec<&&str> = high_pri_by_cat.keys().collect();
    if high_pri_cats.len() >= 2 {
        // e.g. both "applicant" and "developer" have p4+ tasks
        for combo in high_pri_cats.windows(2) {
            conflicts.push(IdentityConflict {
                severity: 0.9,
                message: format!(
                    "Conflicting identities: your '{}' tasks and '{}' tasks \
                     are both competing for your best energy.",
                    combo[0], combo[1]
                ),
                suggestion: format!(
                    "Put all '{}' tasks on one day, '{}' on another.",
                    combo[0], combo[1]
                ),
            });
        }
    }

    // CONFLICT 3: Category switch cost
    // Frequent category alternation within the schedule = context switching penalty
    let mut switches = 0;
    for window in tasks.windows(2) {
        let cat_a = window[0].task.category.as_deref().unwrap_or("uncategorised");
        let cat_b = window[1].task.category.as_deref().unwrap_or("uncategorised");
        if cat_a != cat_b { switches += 1; }
    }
    let switch_rate = switches as f64 / (tasks.len() - 1).max(1) as f64;
    if switch_rate > 0.6 {
        conflicts.push(IdentityConflict {
            severity: 0.6,
            message: format!(
                "You're switching between roles every {:.1} tasks. \
                 That's a lot of context switching.",
                1.0 / switch_rate
            ),
            suggestion: "Group same-category tasks together to reduce mental gear-shifting.".into(),
        });
    }

    conflicts
}
```

---

## 5. Decision Debt Tracking

**Problem:** Every unresolved choice — no deadline, vague priority, uncategorised task — is mental weight the user is carrying without realising it.

### What counts as debt

```rust
fn compute_decision_debt(tasks: &[Task]) -> DecisionDebt {
    let mut debt_items = Vec::new();

    for task in tasks {
        // DEBT 1: No deadline (open-ended obligation)
        if task.deadline.is_none() {
            debt_items.push(DebtItem {
                task_id: task.id.clone(),
                task_name: task.name.clone(),
                kind: DebtKind::NoDeadline,
                message: "No deadline — this is an open loop in your brain".into(),
                weight: 2.0,  // open loops are expensive
            });
        }

        // DEBT 2: Medium priority (unresolved importance)
        if task.priority == 3 {
            debt_items.push(DebtItem {
                task_id: task.id.clone(),
                task_name: task.name.clone(),
                kind: DebtKind::VaguePriority,
                message: "Priority 3 — you haven't decided if this really matters".into(),
                weight: 1.0,
            });
        }

        // DEBT 3: No tags (uncategorised)
        if task.tags.is_empty() {
            debt_items.push(DebtItem {
                task_id: task.id.clone(),
                task_name: task.name.clone(),
                kind: DebtKind::Uncategorised,
                message: "No tags — you haven't thought about what kind of work this is".into(),
                weight: 0.5,
            });
        }

        // DEBT 4: No category (identity ambiguity)
        if task.category.is_none() {
            debt_items.push(DebtItem {
                task_id: task.id.clone(),
                task_name: task.name.clone(),
                kind: DebtKind::NoIdentity,
                message: "No category — which role does this belong to?".into(),
                weight: 0.5,
            });
        }

        // DEBT 5: Vague name
        if task.name.len() < 5 || task.name.to_lowercase().contains("stuff")
            || task.name.to_lowercase().contains("things")
            || task.name.to_lowercase().contains("work on")
        {
            debt_items.push(DebtItem {
                task_id: task.id.clone(),
                task_name: task.name.clone(),
                kind: DebtKind::VagueName,
                message: "Vague task name — what exactly needs to happen?".into(),
                weight: 1.0,
            });
        }
    }

    let total_score: f64 = debt_items.iter().map(|d| d.weight).sum();

    DecisionDebt {
        score: total_score as u32,
        items: debt_items,
        warning: total_score > 10.0,
    }
}
```

### Thresholds

| Debt Score | State | UI |
|------------|-------|----|
| 0–3 | Healthy | Green badge, no alert |
| 4–7 | Moderate | Yellow badge: "Some open decisions" |
| 8–10 | High | Orange badge: "Clear some choices" |
| > 10 | Critical | Red banner: "Decision debt is critical. Resolve before generating." |

### UI

- Navbar badge shows total debt score
- Click → expandable panel listing each debt item with the specific task and what's unresolved
- Each item has a quick-fix link that opens the task form pre-filled for editing

---

## 6. Regret Minimisation Mode

**Problem:** The user picks Serenity or Crunch, but doesn't know if they'll regret the choice. Which tasks get excluded matters more than which get included.

### Algorithm

```rust
async fn schedule_regret_minimisation(
    tasks: Vec<Task>, available_time: u32
) -> Result<Schedule, ServerFnError> {
    // Run both modes
    let serenity = schedule_serenity(tasks.clone(), available_time).await?;
    let crunch = schedule_crunch(tasks.clone(), available_time).await?;

    // For each mode, compute regret for excluded tasks
    let serenity_regret = compute_regret(&tasks, &serenity, available_time);
    let crunch_regret = compute_regret(&tasks, &crunch, available_time);

    // Pick the mode with lower MAXIMUM regret (minimax strategy)
    // This is "regret minimisation" — minimise your worst-case regret
    let serenity_max_regret = serenity_regret.iter()
        .map(|r| r.regret_score).fold(0.0, f64::max);
    let crunch_max_regret = crunch_regret.iter()
        .map(|r| r.regret_score).fold(0.0, f64::max);

    let chosen = if serenity_max_regret <= crunch_max_regret {
        serenity
    } else {
        crunch
    };

    // Also compute regret for the chosen schedule's excluded tasks
    let regret_items = compute_regret(&tasks, &chosen, available_time);

    Ok(Schedule {
        mode: ScheduleMode::RegretMinimisation,
        regret_items,  // attach for UI display
        ..chosen
    })
}

fn compute_regret(
    all_tasks: &[Task], schedule: &Schedule, available_time: u32
) -> Vec<RegretItem> {
    let scheduled_ids: HashSet<&str> = schedule.tasks.iter()
        .map(|t| t.task.id.as_str()).collect();

    all_tasks.iter()
        .filter(|t| !scheduled_ids.contains(t.id.as_str()))
        .map(|task| {
            let hours_until = task.deadline.as_ref()
                .map(|d| hours_between(now(), parse_iso(d)))
                .unwrap_or(72.0);  // no deadline = assume 72h (low urgency)

            let regret = task.priority as f64
                * (1.0 / hours_until.max(1.0))
                * (1.0 + task.emotional_weight);

            RegretItem {
                task_id: task.id.clone(),
                task_name: task.name.clone(),
                regret_score: regret,
                reason: format!(
                    "Excluded despite priority {} — {} hours until deadline, \
                     emotional weight {:.1}",
                    task.priority, hours_until as u32, task.emotional_weight
                ),
            }
        })
        .filter(|r| r.regret_score > 0.5)  // only surface meaningful regret
        .sorted_by(|a, b| b.regret_score.partial_cmp(&a.regret_score).unwrap())
        .take(3)  // top 3 only
        .collect()
}
```

---

## 7. Momentum Engineering

**Problem:** Starting a work session with a 90-minute emotionally heavy task kills motivation. Start with quick wins, build confidence, then tackle hard stuff.

### Algorithm

```rust
fn apply_momentum_ordering(tasks: &mut Vec<ScheduledTask>) -> f64 {
    let n = tasks.len();
    if n < 3 { return 1.0; }  // too few tasks to engineer momentum

    // PHASE 1: Identify confidence builders
    // Score = priority / (emotional_weight * duration)
    // High priority, low weight, short duration = best starter
    let mut scored: Vec<(usize, f64)> = tasks.iter().enumerate().map(|(i, t)| {
        let score = t.task.priority as f64
            / ((t.task.emotional_weight * 0.5 + 0.5) * t.task.duration as f64 / 60.0);
        (i, score)
    }).collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    // PHASE 2: Place top 2–3 confidence builders at the start
    let warmup_count = (n / 4).clamp(1, 3);  // 25% of tasks, min 1, max 3
    let warmup_indices: Vec<usize> = scored.iter()
        .take(warmup_count)
        .map(|(i, _)| *i)
        .collect();

    // Reorder: warmup tasks first, then rest in original order
    let warmup: Vec<ScheduledTask> = warmup_indices.iter()
        .map(|&i| tasks[i].clone())
        .collect();
    let rest: Vec<ScheduledTask> = tasks.iter()
        .enumerate()
        .filter(|(i, _)| !warmup_indices.contains(i))
        .map(|(_, t)| t.clone())
        .collect();

    tasks.clear();
    tasks.extend(warmup);
    tasks.extend(rest);

    // PHASE 3: Recompute start/end times
    let mut time = 0;
    for t in tasks.iter_mut() {
        t.start_time = time;
        time += t.task.duration;
        t.end_time = time;
    }

    // PHASE 4: Compute momentum score
    // = how much easier the first few tasks are vs the average
    let avg_emotional_all: f64 = tasks.iter()
        .map(|t| t.task.emotional_weight).sum::<f64>() / n as f64;
    let avg_emotional_first: f64 = tasks.iter()
        .take(warmup_count)
        .map(|t| t.task.emotional_weight).sum::<f64>() / warmup_count as f64;

    // Higher score = better momentum (easier start relative to average)
    (1.0 - (avg_emotional_first / avg_emotional_all.max(0.01))).clamp(0.0, 1.0)
}
```

### UI

- Momentum gauge: green (≥ 0.6), amber (0.3–0.6), red (< 0.3)
- If red: "Your hardest tasks are up front. Consider rearranging or adding easier tasks to warm up."

---

## 8. Micro-Recovery Insertion

**Problem:** Back-to-back tasks without breaks guarantees fatigue. But the user won't manually insert breaks.

### Algorithm

```rust
fn insert_micro_recoveries(
    schedule: &mut Schedule, config: &RecoveryConfig
) {
    let mut insertions: Vec<(usize, ScheduledTask)> = Vec::new();
    let mut cumulative_work: u32 = 0;

    for (i, scheduled) in schedule.tasks.iter().enumerate() {
        cumulative_work += scheduled.task.duration;

        // RULE 1: After heavy emotional tasks
        if scheduled.task.emotional_weight > config.heavy_threshold {
            insertions.push((i + 1, make_break(config.heavy_break_minutes)));
        }

        // RULE 2: After long tasks
        if scheduled.task.duration > config.long_task_threshold_minutes {
            insertions.push((i + 1, make_break(config.long_break_minutes)));
        }

        // RULE 3: After cumulative work exceeds threshold
        if cumulative_work >= config.cumulative_threshold_minutes {
            insertions.push((i + 1, make_break(config.cumulative_break_minutes)));
            cumulative_work = 0;  // reset counter
        }
    }

    // Insert in reverse order so indices don't shift
    for (pos, break_task) in insertions.into_iter().rev() {
        schedule.tasks.insert(pos.min(schedule.tasks.len()), break_task);
    }

    // Recompute all start/end times
    let mut time = 0;
    for t in schedule.tasks.iter_mut() {
        t.start_time = time;
        time += t.task.duration;
        t.end_time = time;
    }
}

fn make_break(minutes: u32) -> ScheduledTask {
    ScheduledTask {
        task: Task {
            id: format!("break-{}", uuid()),
            name: match minutes {
                5 => "Quick reset".into(),
                10 => "Take a breath".into(),
                15 => "Real break — walk away".into(),
                _ => "Break".into(),
            },
            duration: minutes,
            priority: 0,
            deadline: None,
            tags: vec!["break".into()],
            emotional_weight: 0.0,
            category: Some("recovery".into()),
        },
        start_time: 0,  // recomputed after
        end_time: 0,
        stress_contribution: 0.0,
        deadline_risk: 0.0,
    }
}
```

### Default config

```rust
RecoveryConfig {
    heavy_threshold: 0.6,           // emotional_weight > 0.6 = heavy
    heavy_break_minutes: 5,
    long_task_threshold_minutes: 60,
    long_break_minutes: 5,
    cumulative_threshold_minutes: 120,  // every 2 hours of work
    cumulative_break_minutes: 10,
    enabled: true,  // toggle in config panel
}
```

### UI

- Break blocks in timeline: light blue, coffee/stretch icon
- Breaks do NOT count toward productivity score (they're not "tasks")
- Breaks DO reduce stress index and reduce time utilisation
- Config panel toggle: "Insert recovery breaks" with advanced options expandable

---

## 9. Task Emotional Weighting

**Problem:** A 15-minute "Call the dentist" task and a 15-minute "Reply to rejection email" task take the same time but have completely different psychological cost.

### How it affects scoring

**Serenity Mode (greedy):**
```
// Before (Phase 1):
density = priority / duration

// After (Phase 8):
density = (priority + emotional_weight * 2.0) / duration
// emotional_weight of 1.0 adds +2 to effective priority
// A p3 task with emotional_weight 0.8 scores like a p4.6 task
```

**Crunch Mode (knapsack):**
```
// Before (Phase 1):
crunch_score = priority * (1 + urgency)

// After (Phase 8):
crunch_score = priority * (1 + urgency) * (1 + emotional_weight * 0.5)
// emotional_weight of 1.0 multiplies score by 1.5x
// Under pressure, heavy tasks get weighted MORE (don't leave them until you're exhausted)
```

**Stress Index:**
```
// Before (Phase 1):
stress_contribution = duration * position_weight * priority_weight

// After (Phase 8):
stress_contribution = duration * position_weight * (priority_weight + emotional_weight)
// Emotional weight directly increases a task's stress contribution
// Heavy tasks make the schedule feel heavier even if they're short
```

**Momentum score:**
- Emotional weight is the primary input: low-weight tasks at start = high momentum

**Failure Forecasting:**
- Emotional weight is one of 6 factors in the risk score (20% weight)

### UI

- Slider in task form: 0.0 (Light) — 0.5 (Moderate) — 1.0 (Heavy)
- Default: 0.0 (user doesn't have to think about it)
- Tooltip: "How does this task *feel*? A 15-minute scary phone call is heavier than a 15-minute easy email."

---

## 10. What-If Simulation

**Problem:** The user wants to know "what happens if I skip this?" without actually losing their current schedule.

### Algorithm

```rust
async fn simulate_change(
    original_tasks: Vec<Task>,
    available_time: u32,
    mode: ScheduleMode,
    change: ScheduleChange,
) -> Result<SimulationResult, ServerFnError> {
    // Build modified task list
    let modified_tasks: Vec<Task> = match &change {
        ScheduleChange::RemoveTask(id) =>
            original_tasks.iter().filter(|t| t.id != *id).cloned().collect(),
        ScheduleChange::DelayTask(id) => {
            // Move task to end by setting its deadline further out
            let mut tasks = original_tasks.clone();
            if let Some(t) = tasks.iter_mut().find(|t| t.id == *id) {
                t.deadline = Some(add_hours(now(), 48).to_iso());
            }
            tasks
        }
        ScheduleChange::AddTime(minutes) =>
            original_tasks,  // same tasks, different W
    };

    let modified_w = match &change {
        ScheduleChange::AddTime(extra) => available_time + extra,
        _ => available_time,
    };

    // Generate new schedule
    let new_schedule = match mode {
        ScheduleMode::Serenity =>
            schedule_serenity(modified_tasks, modified_w).await?,
        ScheduleMode::Crunch =>
            schedule_crunch(modified_tasks, modified_w).await?,
        ScheduleMode::RegretMinimisation =>
            schedule_regret_minimisation(modinal_tasks, modified_w).await?,
    };

    // Also need original metrics (recompute to ensure apples-to-apples)
    let original_schedule = match mode {
        ScheduleMode::Serenity =>
            schedule_serenity(original_tasks.clone(), available_time).await?,
        ScheduleMode::Crunch =>
            schedule_crunch(original_tasks.clone(), available_time).await?,
        _ => return Err("Unsupported mode for simulation".into()),
    };

    let diff = MetricsDiff {
        productivity: new_schedule.metrics.productivity_score
            - original_schedule.metrics.productivity_score,
        time_utilisation: new_schedule.metrics.time_utilisation
            - original_schedule.metrics.time_utilisation,
        stress_index: new_schedule.metrics.stress_index
            - original_schedule.metrics.stress_index,
        deadline_risk: new_schedule.metrics.deadline_risk
            - original_schedule.metrics.deadline_risk,
        tasks_scheduled: new_schedule.tasks.len() as i32
            - original_schedule.tasks.len() as i32,
    };

    Ok(SimulationResult {
        new_schedule,
        diff,
        summary: generate_summary(&change, &diff),
    })
}

fn generate_summary(change: &ScheduleChange, diff: &MetricsDiff) -> String {
    let action = match change {
        ScheduleChange::RemoveTask(id) => format!("Dropping task"),
        ScheduleChange::DelayTask(id) => format!("Delaying task"),
        ScheduleChange::AddTime(m) => format!("Adding {} minutes", m),
    };

    let mut effects = Vec::new();
    if diff.stress_index < -0.05 {
        effects.push(format!("stress drops {:.0}%", diff.stress_index.abs() * 100.0));
    }
    if diff.stress_index > 0.05 {
        effects.push(format!("stress rises {:.0}%", diff.stress_index * 100.0));
    }
    if diff.deadline_risk > 0.05 {
        effects.push(format!("deadline risk up {:.0}%", diff.deadline_risk * 100.0));
    }
    if diff.deadline_risk < -0.05 {
        effects.push(format!("deadline risk down {:.0}%", diff.deadline_risk.abs() * 100.0));
    }
    if diff.tasks_scheduled < 0 {
        effects.push(format!("{} fewer task(s)", diff.tasks_scheduled.abs()));
    }

    format!("{}: {}", action, effects.join(", "))
}
```

### UI

- "What if I drop this?" button on each scheduled task
- "What if I delay this?" button on deadline-bearing tasks
- "What if I had more time?" slider on config panel (add 30/60/90 min)
- Inline diff: green for improvements, red for regressions
- Summary sentence: "Dropping Task X: stress drops 15%, deadline risk up 8%, 1 fewer task"

---

## 11. Push Notifications

**Problem:** The user generates a schedule, closes the tab, and forgets about it. The schedule is useless if nobody acts on it at the right time.

### Architecture

```
┌──────────────────────────────────────────────────────────────────────┐
│                         SERVER SIDE                                  │
│                                                                      │
│  schedule_notifications()        Convex Cron (every 1 min)           │
│  ┌─────────────────────┐        ┌─────────────────────────┐         │
│  │ Generate schedule    │        │ Query scheduled_notif.  │         │
│  │ For each task:       │        │ WHERE trigger_at <= now │         │
│  │   create notif at    │───────►│   AND sent == false     │         │
│  │   start_time - 10min │  DB    │ For each:               │         │
│  │ For each break:      │        │   send_push()           │         │
│  │   create notif at    │        │   mark sent = true      │         │
│  │   break end_time     │        └────────────┬────────────┘         │
│  │ + daily summary      │                     │                      │
│  │ + overload warning   │                     │ web-push crate       │
│  └─────────────────────┘                     │ (VAPID signed)       │
│                                               ▼                      │
│                                      Push Service (FCM/APNs/Mozilla) │
└──────────────────────────────────────────────┬───────────────────────┘
                                               │
                                               ▼
┌──────────────────────────────────────────────────────────────────────┐
│                         BROWSER SIDE                                 │
│                                                                      │
│  Service Worker (sw.js)                                              │
│  ┌─────────────────────────────────────────────┐                     │
│  │ self.addEventListener('push', (event) => {  │                     │
│  │   const data = event.data.json();           │                     │
│  │   self.registration.showNotification(       │                     │
│  │     data.title, {                           │                     │
│  │       body: data.body,                      │                     │
│  │       icon: data.icon,                      │                     │
│  │       data: { url, task_id },               │                     │
│  │       actions: data.actions,                │                     │
│  │       tag: data.tag,                        │ ◄── dedup           │
│  │     }                                       │                     │
│  │   );                                        │                     │
│  │ });                                         │                     │
│  │                                             │                     │
│  │ self.addEventListener('notificationclick',  │                     │
│  │   (event) => {                              │                     │
│  │     // focus tab + navigate to schedule     │                     │
│  │     // or scroll to specific task           │                     │
│  │ });                                         │                     │
│  └─────────────────────────────────────────────┘                     │
└──────────────────────────────────────────────────────────────────────┘
```

### Key design decisions

1. **Server schedules, not client** — the browser may be closed when the notification should fire. Server stores the schedule and sends at the right time via push service.
2. **Convex cron triggers delivery** — a scheduled function runs every 60 seconds, checks for due notifications, sends them. No external cron service needed.
3. **VAPID authentication** — Voluntary Application Server Identification. Server proves to the push service that it's authorised to send to a given subscription. Uses ECDSA P-256 key pair.
4. **Tag-based deduplication** — if the user has multiple tabs/devices subscribed, the push service delivers to all. The `tag` field ensures the browser only shows one notification per logical event (e.g., `task-abc-10min`).
5. **Graceful degradation** — if no subscription exists, the app works fine. Notifications are additive, never blocking.

### 11.1 Service Worker (`public/sw.js`)

The service worker is pure JavaScript — it cannot be WASM because it must run in the background even when no page is open.

```javascript
// public/sw.js

const CACHE_NAME = 'agamotto-sw-v1';

// INSTALL: activate immediately
self.addEventListener('install', (event) => {
    self.skipWaiting();
});

// ACTIVATE: claim all open tabs
self.addEventListener('activate', (event) => {
    event.waitUntil(self.clients.claim());
});

// PUSH: show notification from server payload
self.addEventListener('push', (event) => {
    if (!event.data) return;

    const data = event.data.json();

    const options = {
        body: data.body,
        icon: '/assets/favicon.ico',
        badge: '/assets/favicon.ico',
        data: {
            url: data.url || '/',
            task_id: data.task_id || null,
        },
        actions: (data.actions || []).map(a => ({
            action: a.action,
            title: a.title,
        })),
        tag: data.tag || 'agamotto-default',
        renotify: true,          // show even if same tag (update)
        requireInteraction: false, // auto-dismiss after a few seconds
        silent: false,
        vibrate: [200, 100, 200], // mobile vibration pattern
    };

    event.waitUntil(
        self.registration.showNotification(data.title, options)
    );
});

// NOTIFICATION CLICK: focus existing tab or open new one
self.addEventListener('notificationclick', (event) => {
    event.notification.close();

    const { url, task_id } = event.notification.data || {};
    const targetUrl = url || '/';

    event.waitUntil(
        self.clients.matchAll({ type: 'window', includeUncontrolled: true })
            .then((windowClients) => {
                // Try to focus an existing tab with the app
                for (const client of windowClients) {
                    if (client.url.includes(self.location.origin)) {
                        client.focus();
                        // Post message to scroll to task if task_id exists
                        if (task_id) {
                            client.postMessage({
                                type: 'SCROLL_TO_TASK',
                                task_id: task_id,
                            });
                        }
                        return;
                    }
                }
                // No existing tab — open a new one
                return self.clients.openWindow(targetUrl);
            })
    );
});

// PUSH SUBSCRIPTION CHANGE: re-subscribe automatically
// Fires when the push service expires or rotates the subscription
self.addEventListener('pushsubscriptionchange', (event) => {
    event.waitUntil(
        self.registration.pushManager.subscribe(
            event.oldSubscription.options
        ).then((newSubscription) => {
            // Send new subscription to server
            return fetch('/api/push/resubscribe', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    old_endpoint: event.oldSubscription.endpoint,
                    new_subscription: newSubscription.toJSON(),
                }),
            });
        })
    );
});
```

### 11.2 VAPID Key Management

```rust
// Server-side: VAPID key pair generation (one-time setup)
// Generated via: web-push generate-vapid-keys
// Stored as environment variables:
//   VAPID_PUBLIC_KEY  = base64url-encoded public key
//   VAPID_PRIVATE_KEY = base64url-encoded private key
//   VAPID_SUBJECT     = "mailto:admin@agamotto.app"

#[server]
async fn get_vapid_public_key() -> Result<String, ServerFnError> {
    Ok(std::env::var("VAPID_PUBLIC_KEY")
        .expect("VAPID_PUBLIC_KEY not set"))
}
```

```rust
// Client-side: register service worker and subscribe
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{PushManager, PushSubscriptionOptionsInit, ServiceWorkerRegistration};

async fn init_push_notifications() -> Result<(), PushError> {
    let window = web_sys::window().ok_or("No window")?;
    let navigator = window.navigator();

    // 1. Register service worker
    let sw_container = navigator.service_worker();
    let registration = wasm_bindgen_futures::JsFuture::from(
        sw_container.register("/sw.js")
    ).await?;
    let sw_reg: ServiceWorkerRegistration = registration.into();

    // 2. Check if already subscribed
    let push_manager = sw_reg.push_manager();
    let existing = wasm_bindgen_futures::JsFuture::from(
        push_manager.get_subscription()
    ).await?;

    if !existing.is_undefined() && !existing.is_null() {
        // Already subscribed
        return Ok(());
    }

    // 3. Get VAPID public key from server
    let vapid_key = get_vapid_public_key().await?;

    // 4. Subscribe
    let options = PushSubscriptionOptionsInit::new();
    options.set_user_visible_only(true);
    // Decode base64url VAPID key to Uint8Array
    let key_bytes = base64_url_decode(&vapid_key);
    options.set_application_server_key(
        &js_sys::Uint8Array::from(&key_bytes[..])
    );

    let subscription = wasm_bindgen_futures::JsFuture::from(
        push_manager.subscribe(&options)
    ).await?;

    // 5. Send subscription to server
    let sub_json: serde_json::Value = serde_wasm_bindgen::from_value(subscription)?;
    subscribe_to_push(sub_json).await?;

    Ok(())
}

fn base64_url_decode(input: &str) -> Vec<u8> {
    // base64url → base64
    let padded = input
        .replace('-', "+")
        .replace('_', "/");
    let padding = (4 - padded.len() % 4) % 4;
    let padded = format!("{}{}", padded, "=".repeat(padding));
    base64::decode(&padded).unwrap_or_default()
}
```

### 11.3 Subscription Storage

```rust
// Server: store subscription in Convex
#[derive(Clone, Debug, Serialize, Deserialize)]
struct PushSubscription {
    session_id: String,
    endpoint: String,       // e.g. "https://fcm.googleapis.com/fcm/send/..."
    p256dh: String,         // client public key for encryption
    auth: String,           // client auth secret
    created_at: String,     // ISO 8601
}

#[server]
async fn subscribe_to_push(
    subscription: serde_json::Value
) -> Result<(), ServerFnError> {
    let session_id = get_session_id();  // from cookie or generated
    let endpoint = subscription["endpoint"].as_str().unwrap().to_string();
    let keys = &subscription["keys"];
    let p256dh = keys["p256dh"].as_str().unwrap().to_string();
    let auth = keys["auth"].as_str().unwrap().to_string();

    let sub = PushSubscription {
        session_id,
        endpoint,
        p256dh,
        auth,
        created_at: utc_now_iso(),
    };

    // Upsert: one subscription per session+endpoint
    convex_upsert("push_subscriptions", sub).await?;
    Ok(())
}
```

### 11.4 Notification Scheduling Algorithm

```rust
#[server]
async fn schedule_notifications(
    schedule: Schedule,
    session_id: String,
) -> Result<(), ServerFnError> {
    // 1. Check if user has notifications enabled
    let prefs = get_notification_prefs(&session_id).await?;
    if !prefs.enabled { return Ok(()); }

    // Check if snoozed
    if let Some(snoozed_until) = &prefs.snoozed_until {
        if parse_iso(snoozed_until) > now() { return Ok(()); }
    }

    // 2. Get user's push subscription
    let subscription = get_subscription(&session_id).await?;
    if subscription.is_none() { return Ok(()); }
    let sub = subscription.unwrap();

    // 3. Get schedule's absolute start time
    // User says "I'm starting now" or "I'm starting at 9:00 AM"
    let schedule_start = get_schedule_start_time(&session_id).await?;

    let mut notifications = Vec::new();

    // 4. Schedule task reminders (10 min before each task)
    if prefs.task_reminders {
        for scheduled in &schedule.tasks {
            if scheduled.task.priority == 0 { continue; } // skip breaks

            let trigger_at = schedule_start
                + chrono::Duration::minutes(scheduled.start_time as i64)
                - chrono::Duration::minutes(10);

            if trigger_at > now() {
                let urgency_label = if scheduled.task.priority >= 4 {
                    "⚠️ "
                } else {
                    ""
                };

                notifications.push(ScheduledNotification {
                    session_id: session_id.clone(),
                    notification_type: "task_starting".into(),
                    trigger_at: trigger_at.to_iso(),
                    payload: NotificationPayload {
                        title: format!(
                            "{}Starting soon: {}",
                            urgency_label, scheduled.task.name
                        ),
                        body: format!(
                            "{} min · Priority {}/5",
                            scheduled.task.duration,
                            scheduled.task.priority
                        ),
                        url: Some("/schedule".into()),
                        task_id: Some(scheduled.task.id.clone()),
                        tag: format!("task-{}-10min", scheduled.task.id),
                        actions: vec![
                            NotificationAction {
                                action: "view".into(),
                                title: "View Schedule".into(),
                            },
                        ],
                    },
                    sent: false,
                    subscription_endpoint: sub.endpoint.clone(),
                });
            }
        }
    }

    // 5. Schedule break-end reminders
    if prefs.break_reminders {
        for (i, scheduled) in schedule.tasks.iter().enumerate() {
            if scheduled.task.priority != 0 { continue; } // only breaks

            let break_end = schedule_start
                + chrono::Duration::minutes(scheduled.end_time as i64);

            if break_end > now() {
                // Find the next task after this break
                let next_task = schedule.tasks.get(i + 1)
                    .map(|t| t.task.name.as_str())
                    .unwrap_or("nothing — you're done!");

                notifications.push(ScheduledNotification {
                    session_id: session_id.clone(),
                    notification_type: "break_over".into(),
                    trigger_at: break_end.to_iso(),
                    payload: NotificationPayload {
                        title: "Break's over".into(),
                        body: format!("Next up: {}", next_task),
                        url: Some("/schedule".into()),
                        task_id: schedule.tasks.get(i + 1)
                            .map(|t| t.task.id.clone()),
                        tag: format!("break-{}-end", scheduled.task.id),
                        actions: vec![],
                    },
                    sent: false,
                    subscription_endpoint: sub.endpoint.clone(),
                });
            }
        }
    }

    // 6. Schedule deadline alerts (1 hour before each task deadline)
    if prefs.deadline_alerts {
        for scheduled in &schedule.tasks {
            if let Some(deadline) = &scheduled.task.deadline {
                let deadline_dt = parse_iso(deadline);
                let alert_at = deadline_dt - chrono::Duration::hours(1);

                if alert_at > now() {
                    notifications.push(ScheduledNotification {
                        session_id: session_id.clone(),
                        notification_type: "deadline_alert".into(),
                        trigger_at: alert_at.to_iso(),
                        payload: NotificationPayload {
                            title: "⏰ Deadline in 1 hour".into(),
                            body: format!(
                                "{} — due at {}",
                                scheduled.task.name,
                                deadline_dt.format("%H:%M")
                            ),
                            url: Some("/schedule".into()),
                            task_id: Some(scheduled.task.id.clone()),
                            tag: format!("deadline-{}", scheduled.task.id),
                            actions: vec![
                                NotificationAction {
                                    action: "view".into(),
                                    title: "View".into(),
                                },
                            ],
                        },
                        sent: false,
                        subscription_endpoint: sub.endpoint.clone(),
                    });
                }
            }
        }
    }

    // 7. Schedule daily summary
    if prefs.daily_summary {
        let summary_time = parse_time_today(&prefs.daily_summary_time);
        // If summary time has passed today, schedule for tomorrow
        let trigger = if summary_time > now() {
            summary_time
        } else {
            summary_time + chrono::Duration::days(1)
        };

        let task_count = schedule.tasks.iter()
            .filter(|t| t.task.priority > 0).count();
        let first_task = schedule.tasks.iter()
            .find(|t| t.task.priority > 0)
            .map(|t| t.task.name.as_str())
            .unwrap_or("nothing scheduled");

        notifications.push(ScheduledNotification {
            session_id: session_id.clone(),
            notification_type: "daily_summary".into(),
            trigger_at: trigger.to_iso(),
            payload: NotificationPayload {
                title: "📋 Your day ahead".into(),
                body: format!(
                    "{} tasks today. First up: {}",
                    task_count, first_task
                ),
                url: Some("/schedule".into()),
                task_id: None,
                tag: "daily-summary".into(),
                actions: vec![
                    NotificationAction {
                        action: "view".into(),
                        title: "View Schedule".into(),
                    },
                ],
            },
            sent: false,
            subscription_endpoint: sub.endpoint.clone(),
        });
    }

    // 8. Immediate: overload warning
    if prefs.overload_warnings && schedule.metrics.overload_flag {
        notifications.push(ScheduledNotification {
            session_id: session_id.clone(),
            notification_type: "overload_warning".into(),
            trigger_at: utc_now_iso(),  // send immediately
            payload: NotificationPayload {
                title: "⚠️ Schedule overload detected".into(),
                body: "This schedule might be too heavy. Consider dropping a task.".into(),
                url: Some("/schedule".into()),
                task_id: None,
                tag: "overload-warning".into(),
                actions: vec![],
            },
            sent: false,
            subscription_endpoint: sub.endpoint.clone(),
        });
    }

    // 9. Save all notifications to Convex
    for notif in notifications {
        convex_insert("scheduled_notifications", notif).await?;
    }

    Ok(())
}
```

### 11.5 Push Delivery (Convex Cron)

```rust
// Runs every 60 seconds via Convex scheduled functions
async fn deliver_pending_notifications() -> Result<u32, ServerFnError> {
    let now = utc_now_iso();

    // 1. Query undelivered notifications whose time has passed
    let pending: Vec<ScheduledNotification> = convex_query(
        "scheduled_notifications",
        |q| q
            .filter(|n| n.field("sent").eq(false))
            .filter(|n| n.field("trigger_at").lte(now))
            .order("trigger_at", "asc")
            .limit(50)  // batch size
    ).await?;

    let mut delivered = 0;

    for notif in &pending {
        // 2. Look up subscription
        let sub = get_subscription_by_endpoint(
            &notif.subscription_endpoint
        ).await?;

        if let Some(subscription) = sub {
            // 3. Send push
            match send_push(
                &subscription,
                &notif.payload,
            ).await {
                Ok(_) => {
                    // Mark sent
                    mark_notification_sent(&notif.id).await?;
                    delivered += 1;
                }
                Err(PushError::SubscriptionExpired) => {
                    // 410 Gone — subscription expired
                    remove_subscription(&notif.subscription_endpoint).await?;
                    // Also remove this notification
                    delete_notification(&notif.id).await?;
                }
                Err(e) => {
                    // Log error but don't block other notifications
                    eprintln!("Push send failed for {}: {:?}", notif.id, e);
                }
            }
        } else {
            // Subscription no longer exists
            delete_notification(&notif.id).await?;
        }
    }

    // 5. Cleanup: delete notifications older than 24 hours
    let cutoff = (now() - chrono::Duration::hours(24)).to_iso();
    convex_delete_where("scheduled_notifications", |n| {
        n.filter(|n| n.field("trigger_at").lt(cutoff))
    }).await?;

    Ok(delivered)
}

async fn send_push(
    subscription: &PushSubscription,
    payload: &NotificationPayload,
) -> Result<(), PushError> {
    let content = serde_json::to_string(payload)?;

    let mut builder = WebPushMessageBuilder::new(
        &subscription.endpoint,
        &subscription.p256dh,
        &subscription.auth,
    )?;

    let vapid_sig = VapidSignatureBuilder::from_base64(
        &std::env::var("VAPID_PRIVATE_KEY")?,
        &std::env::var("VAPID_SUBJECT")?,
    )?.build()?;

    builder.set_vapid_signature(vapid_sig);
    builder.set_payload(content.as_bytes());
    builder.set_ttl(60 * 60);  // 1 hour TTL

    let response = builder.send().await?;

    match response.status_code() {
        200..=299 => Ok(()),
        410 => Err(PushError::SubscriptionExpired),
        429 => Err(PushError::RateLimited),
        status => Err(PushError::HttpError(status)),
    }
}
```

### 11.6 Notification Preferences

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
struct NotificationPrefs {
    session_id: String,
    enabled: bool,                // master toggle
    task_reminders: bool,         // 10 min before task
    break_reminders: bool,        // break end
    daily_summary: bool,          // morning overview
    daily_summary_time: String,   // "08:00"
    overload_warnings: bool,      // immediate
    habit_drift_alerts: bool,     // on detection
    deadline_alerts: bool,        // 1 hour before
    snoozed_until: Option<String>, // ISO 8601
}

impl Default for NotificationPrefs {
    fn default() -> Self {
        Self {
            session_id: String::new(),
            enabled: true,
            task_reminders: true,
            break_reminders: true,
            daily_summary: false,  // opt-in (less intrusive)
            daily_summary_time: "08:00".into(),
            overload_warnings: true,
            habit_drift_alerts: true,
            deadline_alerts: true,
            snoozed_until: None,
        }
    }
}
```

### 11.7 UI Flow

**First visit (or after schedule generation):**

```
┌──────────────────────────────────────────────────────┐
│ 🔔 Get notified when it's time to start each task   │
│                                                      │
│ [Enable Notifications]  [Not Now]  [Never]           │
│                                                      │
│ Your schedule only works if you act on it.           │
│ Notifications fire even when this tab is closed.     │
└──────────────────────────────────────────────────────┘
```

- "Enable" → browser permission prompt → subscribe → stored
- "Not Now" → `localStorage.setItem('agamotto-notif-dismissed', Date.now())` → don't ask for 7 days
- "Never" → `localStorage.setItem('agamotto-notif-never', 'true')` → never ask

**Settings panel (navbar gear icon → Notifications):**

```
┌──────────────────────────────────────────────────────┐
│ Notifications                                 [✕]    │
│                                                      │
│ [✓] Enable notifications (master)                    │
│                                                      │
│ Task reminders        [✓] 10 min before each task    │
│ Break reminders       [✓] When break ends            │
│ Daily summary         [ ] Every day at [08:00 ▾]     │
│ Overload warnings     [✓] When schedule is too heavy │
│ Habit drift alerts    [✓] When behaviour changes     │
│ Deadline alerts       [✓] 1 hour before deadline     │
│                                                      │
│ [Snooze all for 2 hours]                             │
│ [Send test notification]                             │
│                                                      │
│ Status: ● Active (Chrome, 1 device)                  │
│ [Unsubscribe]                                        │
└──────────────────────────────────────────────────────┘
```

### 11.8 Edge Cases

| Scenario | Handling |
|----------|----------|
| Permission denied | Show "Notifications blocked. [Instructions for Chrome/Firefox/Safari]" |
| Subscription expired (410) | Silently re-subscribe on next page load |
| No subscription | App works fine, no notifications. Prompt shown after first schedule. |
| Multiple tabs/devices | Push sent to all subscriptions. `tag` field deduplicates within each device. |
| Service worker update | `sw.js` versioned. `skipWaiting` + `clients.claim` on install. |
| Offline | Push service queues for up to TTL (1 hour). If still offline, dropped. |
| Timezone | All `trigger_at` stored as UTC. Service worker uses `Date.now()` (local). |
| Schedule rescheduled | Old notifications deleted via `session_id`, new ones created. |
| User changes system clock | Server-side cron is authoritative. Client clock doesn't affect delivery. |

### 11.9 Files

```
public/
├── sw.js                           # Service worker (push, notificationclick, pushsubscriptionchange)

src/server/
├── push.rs                         # subscribe, unsubscribe, schedule_notifications,
│                                   # deliver_pending_notifications, send_push, prefs

src/components/
├── notification_prompt.rs          # Permission request banner (first visit / post-generate)
├── notification_settings.rs        # Preferences panel with per-type toggles
```

---

## Summary: Detection Trigger Matrix

| Feature | When it runs | Input | Output |
|---------|-------------|-------|--------|
| Habit Drift | Home page load | 14+ behaviour log entries | DriftAlert |
| Invisible Overload | After schedule generation | Schedule + metrics | bool (overload_flag) |
| Failure Forecasting | After schedule generation | Schedule with positions | Vec\<FailurePoint\> |
| Identity Conflict | After schedule generation | Scheduled tasks with categories | Vec\<IdentityConflict\> |
| Decision Debt | Task list changes | All tasks | DecisionDebt { score, items } |
| Regret Minimisation | On generate (Regret mode) | Tasks + W | Schedule + regret_items |
| Momentum Engineering | After task selection | Selected tasks | Reordered tasks + momentum_score |
| Micro-Recovery | After ordering | Ordered tasks + config | Tasks with breaks inserted |
| Emotional Weighting | During scoring | Task.emotional_weight | Modified density/crunch/stress |
| What-If Simulation | On user click | Original schedule + change | SimulationResult + diff |
| Push Notifications | On schedule generate + cron every 1min | Schedule + preferences + subscriptions | Push payloads sent to browser |
