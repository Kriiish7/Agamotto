# Agamotto Roadmap

> **A Constraint-Aware Intelligent Scheduling System**
> Rust + Dioxus 0.7 Fullstack Web App + Convex.dev

---

## Architecture

```
Browser (WASM)                    Server (Rust)                  Convex.dev
┌─────────────────┐  HTTP/IPC    ┌─────────────────┐            ┌───────────┐
│  Dioxus UI       │ ──────────► │  Server Functions │ ────────► │  Database  │
│  - Task Form     │             │  - schedule_ser.. │           │  - tasks   │
│  - Timeline      │             │  - schedule_cru.. │           │  - saves   │
│  - Metrics       │  ◄──────── │  - save/load      │ ◄──────── │            │
│  - Drag reorder  │  JSON       │  - compute_metrics│  JSON     │            │
└─────────────────┘             └─────────────────┘            └───────────┘
```

- **Client**: Dioxus compiles to WASM — handles all UI rendering, form state, drag-and-drop, route navigation
- **Server**: Dioxus fullstack server functions — scheduling engine runs here, Convex client lives here
- **Persistence**: Convex.dev cloud database — accessed server-side only via async Rust SDK

---

## Current State

- Fresh Dioxus 0.7 scaffold with boilerplate `main.rs` (Home, Blog, Echo components)
- Cargo features: `web`, `server` (fullstack)
- No scheduling logic, no custom components, no persistence

---

## Phase 1: Core Types & Server-Side Engine

> **Goal:** Build the scheduling engine as server functions. Engine code never ships to the browser.

### 1.1 Core Data Structures

Define shared types used by both server and client:

- [ ] `src/types.rs` — `Task`, `Schedule`, `ScheduledTask`, `ScheduleMetrics`, `ScheduleMode` structs
  - Types must derive `Clone`, `PartialEq`, `Serialize`, `Deserialize` (serde) for server/client boundary
  - Use `String` IDs (not `Uuid`) to keep types simple across the IPC boundary
- [ ] `src/error.rs` — `ScheduleError` enum for validation errors

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Task {
    id: String,
    name: String,
    duration: u32,             // minutes, 1-480
    priority: u8,              // 1-5
    deadline: Option<String>,  // ISO 8601 string (avoids chrono WASM issues)
    tags: Vec<String>,
    emotional_weight: f64,     // 0.0–1.0, psychological heaviness (Phase 8: Emotional Weighting)
    category: Option<String>,  // e.g. "student", "developer", "applicant" (Phase 8: Identity Conflict)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
enum ScheduleMode { Serenity, Crunch }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Schedule {
    mode: ScheduleMode,
    tasks: Vec<ScheduledTask>,
    available_time: u32,
    metrics: ScheduleMetrics,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ScheduledTask {
    task: Task,
    start_time: u32,
    end_time: u32,
    stress_contribution: f64,
    deadline_risk: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ScheduleMetrics {
    productivity_score: f64,
    time_utilisation: f64,
    stress_index: f64,
    deadline_risk: f64,
    // Phase 8: Future-ready metrics
    overload_flag: bool,            // Invisible Overload Detection
    failure_points: Vec<FailurePoint>, // Failure Forecasting
    decision_debt: u32,             // Decision Debt Tracking
    identity_conflicts: Vec<String>, // Identity Conflict Alerts
    momentum_score: f64,            // Momentum Engineering
    habit_drift_alert: bool,        // Habit Drift Detection
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct FailurePoint {
    task_id: String,
    reason: String,      // e.g. "High stress + tight deadline + late in schedule"
    risk_score: f64,     // 0.0–1.0
}
```

### 1.2 Server Functions — Scheduling Engine

Each algorithm is exposed as a `#[server]` function. The client calls them like normal async functions; Dioxus handles the HTTP transport.

- [ ] `src/server/greedy.rs` — `#[server] async fn schedule_serenity(tasks: Vec<Task>, available_time: u32) -> Result<Schedule, ServerFnError>`
  - Compute priority-density: `score = priority / duration`
  - Sort descending by density, greedily select within time budget
  - Re-sort selected by ascending duration (shortest-first for early wins)
  - Unit tests: GT-01 through GT-05
  - > **Phase 8 hooks:** Density score will later factor in `emotional_weight` (Emotional Weighting). Shortest-first ordering is the foundation for Momentum Engineering (quick wins first). Selection will be extended with Regret Minimisation scoring.

- [ ] `src/server/knapsack.rs` — `#[server] async fn schedule_crunch(tasks: Vec<Task>, available_time: u32) -> Result<Schedule, ServerFnError>`
  - Compute urgency: `urgency = 1 / max(1, hours_until_deadline)`
  - Compute crunch score: `priority × (1 + urgency)`
  - 0/1 Knapsack DP: `dp[n+1][W+1]` table, O(n·W)
  - Backtrack to reconstruct optimal task subset
  - Unit tests: KT-01 through KT-05
  - > **Phase 8 hooks:** Crunch score will later incorporate `emotional_weight` so psychologically heavy tasks get weighted more aggressively under pressure. DP backtracking feeds into Failure Forecasting (identifying where the optimal set is fragile).

- [ ] `src/server/edf.rs` — `order_by_edf(tasks: &mut [ScheduledTask])`
  - Sort by deadline ascending; no-deadline tasks at end sorted by priority descending

- [ ] `src/server/metrics.rs` — `compute_metrics(schedule: &mut Schedule)`
  - Productivity score: `Σ(p_i) / Σ(p_all) × 100%`
  - Time utilisation: `Σ(d_i) / W × 100%`
  - Stress index: weighted sum (position × priority × duration) / (W × max_p), clamped to [0,1]
  - Deadline risk: cumulative completion time vs each task's deadline
  - Unit tests: MT-01 through MT-04
  - > **Phase 8 hooks:** Stress index will later weight tasks by `emotional_weight`. Metrics output will be extended with overload detection, failure forecasting, decision debt, identity conflicts, momentum score, and habit drift alerts.

### 1.3 Server Functions — Persistence

- [ ] `src/server/persistence.rs`
  - `#[server] async fn save_schedule(name: String, schedule: Schedule) -> Result<String, ServerFnError>`
  - `#[server] async fn load_schedule(id: String) -> Result<Schedule, ServerFnError>`
  - `#[server] async fn list_schedules() -> Result<Vec<(String, String)>, ServerFnError>` — returns (id, name) pairs
  - All Convex calls happen here — the browser never talks to Convex directly

### 1.4 Engine Integration Tests

- [ ] End-to-end Serenity: 10 tasks → schedule → metrics in valid ranges
- [ ] End-to-end Crunch: deadline tasks → EDF ordering verified
- [ ] Mode switching produces different subsets
- [ ] Edge cases: empty list, zero time, oversized task, 50-task input

---

## Phase 2: UI Shell & Routing

> **Goal:** Replace boilerplate with Agamotto's two-page web app structure.

### 2.1 Clean Boilerplate

- [ ] Remove Blog, Echo, Hero components from `src/main.rs`
- [ ] Remove `HEADER_SVG`, blog-related constants
- [ ] Keep `App` root component with `Router`

### 2.2 Route Structure

```rust
#[derive(Debug, Clone, Routable, PartialEq)]
enum Route {
    #[route("/")]
    Home {},           // Task input + config + generate
    #[route("/schedule")]
    ScheduleView {},   // Timeline + metrics + reshuffle + save
}
```

- [ ] Define `Route` enum as above
- [ ] `src/components/navbar.rs` — simple top nav with Agamotto branding, links to Home and Schedule

### 2.3 Global State

- [ ] `src/state.rs` — shared `Signal` types for app-wide state:
  - `Signal<Vec<Task>>` — current task list
  - `Signal<Option<Schedule>>` — generated schedule (None until first generation)
  - `Signal<u32>` — available time window in minutes
  - `Signal<ScheduleMode>` — selected mode

### 2.4 Assets

- [ ] `assets/main.css` — base styles (layout, typography, form elements)
- [ ] `assets/tailwind.css` — Tailwind utility classes
- [ ] `assets/favicon.ico` — app icon

---

## Phase 3: Task Input Page (Home)

> **Goal:** User can enter tasks, configure time window, select mode, and generate a schedule.

### 3.1 Task Form Component

- [ ] `src/components/task_form.rs`
  - Name: `<input type="text">`, max 100 chars, required
  - Duration: `<input type="range" min="1" max="480">` with numeric display, step 5
  - Priority: five clickable stars (CSS-only, no JS library) — or `<select>` with labels:
    - 5 = Critical, 4 = High, 3 = Medium, 2 = Low, 1 = Nice to do
  - Deadline: `<input type="datetime-local">`, optional
  - Tags: comma-separated `<input type="text">`, optional
  - Category: `<select>` with options (student, developer, applicant, personal, health, none) — Phase 8: Identity Conflict
  - Emotional weight: `<input type="range" min="0" max="100" step="10">` with labels Light/Moderate/Heavy — Phase 8: Emotional Weighting
  - Add Task button → pushes to `Signal<Vec<Task>>`
  - Client-side validation: name non-empty, duration > 0, priority 1-5

### 3.2 Task List Component

- [ ] `src/components/task_list.rs`
  - Scrollable list of entered tasks
  - Each row: name, duration (formatted as "Xh Ym"), priority stars, deadline (if set), tags
  - Inline edit button → populates form with task data for editing
  - Delete button → removes from signal
  - Empty state message when no tasks entered

### 3.3 Configuration Panel

- [ ] `src/components/config_panel.rs`
  - Time window: hours `<input type="number" min="0" max="8">` + minutes `<input type="number" min="0" max="59" step="15">`
  - Mode toggle: two radio-style buttons (Serenenity / Crunch) with descriptions
    - Serenity: "Relaxed scheduling — quick wins first"
    - Crunch: "Deadline-driven — optimal task selection"
  - Generate Schedule button:
    - Disabled when task list is empty
    - Calls `schedule_serenity()` or `schedule_crunch()` server function
    - Shows loading spinner during async call
    - On success: stores result in `Signal<Option<Schedule>>`, navigates to `/schedule`
    - On error: shows inline error message

### 3.4 Home Page Layout

- [ ] `src/components/home.rs` — combines form + list + config in a responsive grid
  - Left column: task form + task list
  - Right column: config panel + generate button
  - On mobile/narrow viewport: stacked vertically

---

## Phase 4: Schedule Output Page

> **Goal:** Visualise the generated schedule with timeline, metrics, excluded tasks, and reshuffling.

### 4.1 Page Guard

- [ ] If `Signal<Option<Schedule>>` is None, redirect to Home or show "No schedule generated" message
- [ ] Back to Home link to add more tasks or regenerate

### 4.2 Timeline Visualisation

- [ ] `src/components/timeline.rs`
  - Horizontal bar chart inside a scrollable `<div>`
  - Each task rendered as a `<div>` block:
    - Width = `(task.duration / schedule.available_time) × 100%`
    - Background color by priority: green (#22c55e, p1-2), amber (#f59e0b, p3), red (#ef4444, p4-5)
    - Label: task name + time range ("Task A — 0:00–0:30")
  - CSS `position: relative` container; blocks laid out with flexbox
  - Hover: CSS tooltip (`:hover .tooltip`) showing full details

### 4.3 Analytics Dashboard

- [ ] `src/components/metrics_panel.rs`
  - Four cards in a responsive grid (2×2 on desktop, stacked on mobile):
    1. **Productivity Score** — percentage bar, color: blue
    2. **Time Utilisation** — percentage bar, color: indigo
    3. **Stress Index** — bar with thresholds: <0.4 green, 0.4–0.7 amber, >0.7 red
    4. **Deadline Risk** — percentage bar, color: red
  - Each card: label, numeric value, visual bar, tooltip explaining the metric

### 4.4 Excluded Tasks Panel

- [ ] `src/components/excluded_list.rs`
  - List tasks not in the schedule
  - Each shows: task name, reason for exclusion ("Insufficient time — needs 60min, only 15min remaining")
  - Collapsible section (default expanded)

### 4.5 Drag-and-Drop Reshuffling

- [ ] Use HTML5 drag-and-drop API (`draggable="true"`, `ondragstart`, `ondragover`, `ondrop`) on scheduled task rows
  - Drag handle icon on each task row
  - Drop target highlight (dashed border on valid drop zone)
  - On drop: reorder `Vec<ScheduledTask>` in signal, recalculate start/end times
  - Recompute metrics after reorder (stress index and deadline risk are order-dependent)
  - Smooth CSS transition on reorder

- [ ] `src/components/schedule_list.rs` — numbered list view with drag handles
  - Each row: drag handle, number, task name, duration, priority, time range, deadline risk indicator
  - Toggle button to switch between timeline view and list view

### 4.6 Save Schedule

- [ ] Save button in schedule page header
  - Text input for schedule name (pre-filled with timestamp)
  - Calls `save_schedule()` server function
  - Shows success toast on save
  - Shows error toast on failure

### 4.7 Load Schedule

- [ ] `src/components/saved_schedules.rs` — dropdown or sidebar listing saved schedules
  - Fetched via `list_schedules()` on page load
  - Click to load → calls `load_schedule()` → populates signals → renders schedule
  - Delete button per saved schedule

---

## Phase 5: Persistence — Convex Integration

> **Goal:** Convex schema defined, server functions wired, save/load works end-to-end.

### 5.1 Convex Setup

- [ ] `convex/` directory with Convex project init (`npx convex dev`)
- [ ] `convex/schema.ts` — define tables:
  ```typescript
  export default defineSchema({
    tasks: defineTable({
      sessionId: v.string(),
      name: v.string(),
      duration: v.number(),
      priority: v.number(),
      deadline: v.optional(v.string()),
      tags: v.array(v.string()),
    }),
    schedules: defineTable({
      sessionId: v.string(),
      name: v.string(),
      mode: v.string(),
      available_time: v.number(),
      tasks_json: v.string(),  // serialised ScheduledTask array
      metrics_json: v.string(), // serialised ScheduleMetrics
    }),
    // Phase 8: Habit Drift Detection
    behaviour_log: defineTable({
      sessionId: v.string(),
      timestamp: v.string(),
      tasks_planned: v.number(),
      tasks_completed: v.number(),
      avg_priority_selected: v.number(),
      mode_used: v.string(),
      emotional_weight_avg: v.number(),
    }),
    // Phase 8: Push Notifications
    push_subscriptions: defineTable({
      sessionId: v.string(),
      endpoint: v.string(),
      p256dh: v.string(),
      auth: v.string(),
      created_at: v.string(),
    }),
    scheduled_notifications: defineTable({
      sessionId: v.string(),
      notification_type: v.string(),  // "task_starting", "break_over", "daily_summary", etc.
      trigger_at: v.string(),         // ISO 8601 UTC
      payload_json: v.string(),       // serialised NotificationPayload
      sent: v.boolean(),
      subscription_endpoint: v.string(),
    }),
    notification_preferences: defineTable({
      sessionId: v.string(),
      enabled: v.boolean(),
      task_reminders: v.boolean(),
      break_reminders: v.boolean(),
      daily_summary: v.boolean(),
      daily_summary_time: v.string(), // "08:00"
      overload_warnings: v.boolean(),
      habit_drift_alerts: v.boolean(),
      deadline_alerts: v.boolean(),
      snoozed_until: v.optional(v.string()),
    }),
  });
  ```

### 5.2 Convex Client in Server Functions

- [ ] Use the official [`convex`](https://crates.io/crates/convex) Rust SDK (v0.10.3)
  - `ConvexClient::new(&deployment_url).await?` — connects to Convex deployment
  - `client.query("functionName", args).await?` — calls Convex query functions
  - `client.mutation("functionName", args).await?` — calls Convex mutation functions
  - `client.subscribe("functionName", args).await?` — subscribes to live query updates
  - Args are `BTreeMap<String, Value>` where `Value` is the Convex value enum
  - Returns `FunctionResult` — handle `Value` / `ConvexError` variants
- [ ] `src/server/convex_client.rs` — thin wrapper around `ConvexClient` for app-specific operations
  - Reads `CONVEX_URL` from environment (`.env.local` via `dotenvy`)
  - Provides typed helper methods: `insert_task()`, `query_schedules()`, etc.
  - Converts between our `Task`/`Schedule` types and `BTreeMap<String, Value>`
- [ ] Convex functions live in `convex/` directory as TypeScript (queries, mutations, actions)
  - The Rust SDK calls these functions by name — backend logic is defined in TS, called from Rust

### 5.3 Wire Server Functions

- [ ] `save_schedule` → serialise schedule to JSON → insert into `schedules` table
- [ ] `load_schedule` → query `schedules` by ID → deserialise → return `Schedule`
- [ ] `list_schedules` → query `schedules` by session ID → return name + ID pairs

### 5.4 Integration Tests

- [ ] Save a schedule, load it back, verify all task data and metrics match
- [ ] List schedules returns correct entries
- [ ] 5 save/reload cycles preserve data integrity

---

## Phase 6: Polish, Edge Cases & Responsive Design

> **Goal:** Robust, responsive, production-quality web app.

### 6.1 Edge Case Handling

- [ ] Zero available time → empty schedule, informative message "Add some time to get started"
- [ ] All deadlines expired → schedule with 100% deadline risk, warning banner
- [ ] Single task exceeding W → excluded with "Needs X min, only Y available"
- [ ] 50 tasks → schedule generated in < 500ms (server round-trip included)
- [ ] Duplicate priorities → deterministic tie-break by name (alphabetical)
- [ ] Server function timeout → show "Taking too long, try fewer tasks" after 5s

### 6.2 Responsive Design

- [ ] Breakpoints: mobile (<768px), tablet (768-1024px), desktop (>1024px)
- [ ] Home page: side-by-side on desktop, stacked on mobile/tablet
- [ ] Timeline: horizontal scroll on narrow screens
- [ ] Metrics grid: 2×2 on desktop, 1×4 stacked on mobile
- [ ] Task list: full table on desktop, card layout on mobile
- [ ] Nav: horizontal links on desktop, hamburger menu on mobile

### 6.3 UI Polish

- [ ] Priority scale labels visible: "5 = Critical" ... "1 = Nice to do"
- [ ] Loading spinners on all server function calls
- [ ] Smooth CSS transitions on route changes
- [ ] Form validation messages inline (not alerts)
- [ ] Empty states with helpful copy for every list/panel
- [ ] Keyboard accessibility: tab through form, Enter to submit

### 6.4 Performance Targets

| Metric | Target |
|--------|--------|
| Schedule generation (server) | < 100ms for n=50 |
| Full round-trip (generate + render) | < 500ms |
| WASM bundle size | < 2MB gzipped |
| First contentful paint | < 1s on 3G |
| Drag-and-drop frame time | < 16ms (60fps) |

---

## Phase 7: Testing & Evaluation

> **Goal:** Full test coverage, client validation, NEA evidence.

### 7.1 Unit Tests (23 cases)

| Suite | Tests | Coverage |
|-------|-------|----------|
| GT (Greedy) | GT-01 to GT-05 | Priority-density selection, shortest-first ordering |
| KT (Knapsack) | KT-01 to KT-05 | DP table construction, backtracking, performance |
| MT (Metrics) | MT-01 to MT-04 | All four metrics against hand-calculated reference |

- [ ] All 23 unit tests passing via `cargo test`

### 7.2 Integration Tests (5 cases)

| ID | Description |
|----|-------------|
| IT-01 | End-to-end Serenity: 10 tasks → valid schedule + metrics |
| IT-02 | End-to-end Crunch: deadline tasks → EDF order verified |
| IT-03 | Save + reload: data integrity preserved |
| IT-04 | Mode switch: different outputs for same input |
| IT-05 | Empty task list: graceful error, no crash |

### 7.3 System Tests (4 scenarios)

| Scenario | Steps | Expected |
|----------|-------|----------|
| Student revision | 8 tasks, W=240, Serenity | 4-5 tasks, timeline + metrics shown |
| Developer sprint | 12 tasks with deadlines, W=480, Crunch | Optimal subset in EDF order |
| Manual reshuffle | Generate → drag task 3 to position 1 | Order updates, metrics recompute |
| Large input | 50 tasks, W=480, Crunch | Generated in < 500ms |

### 7.4 Browser Compatibility

- [ ] Chrome/Edge (latest) — primary target
- [ ] Firefox (latest)
- [ ] Safari (latest)

### 7.5 Client Evaluation

- [ ] Primary client: 2-week trial, daily use for revision scheduling
- [ ] 3 fellow students: usability testing, feedback forms
- [ ] CS teacher: technical review of code + algorithms
- [ ] Document all feedback and iterations

---

## Phase 8: Future-Ready Features

> **Goal:** Transform Agamotto from a scheduling tool into an intelligent, psychologically-aware productivity system. These features extend the core engine built in Phases 1–7.
>
> **Full algorithmic specifications:** See [`PHASE8_ALGORITHMS.md`](./PHASE8_ALGORITHMS.md) for concrete detection math, scoring formulas, thresholds, and pseudocode for every feature.

### 8.1 Regret Minimisation Mode

Forces the user to confront the consequences of *not* doing a task before it's too late.

- [ ] `src/server/regret.rs` — `compute_regret_scores(tasks, available_time) -> Vec<RegretScore>`
  - For each excluded task, compute: `regret = priority × (1 / hours_until_deadline) × (1 + emotional_weight)`
  - Tasks with high regret scores are flagged: "If you skip this, you'll wish you hadn't"
- [ ] `src/components/regret_panel.rs` — sidebar panel on schedule output page
  - Shows top 3 regret-risk tasks with explanation
  - "Swap In" button: replace lowest-value scheduled task with high-regret task, re-run engine
- [ ] Extend `ScheduleMode` enum: add `RegretMinimisation` variant
- [ ] Server function: `#[server] async fn schedule_regret_minimisation(tasks, time) -> Result<Schedule, ServerFnError>`
  - Runs both Serenity and Crunch internally, compares outputs, highlights the choice that minimises maximum regret

### 8.2 Invisible Overload Detection

Flags schedules that fit the time window but will feel crushing in practice.

- [ ] Extend `compute_metrics` to set `overload_flag`
  - Heuristic: `overload = stress_index > 0.65 AND time_utilisation > 0.9 AND avg_emotional_weight > 0.5`
  - Also flag if ≥3 high-emotional-weight tasks are back-to-back with no gap
- [ ] `src/components/overload_banner.rs` — dismissable warning banner on schedule page
  - "This schedule fits your time, but it might break you. Consider dropping or splitting a task."
  - Suggests which task to remove (highest emotional weight, lowest priority)
- [ ] Unit tests: overloaded schedule detection, false positive prevention

### 8.3 Failure Forecasting

Predicts where the plan is most likely to collapse before the user gets there.

- [ ] `src/server/forecast.rs` — `forecast_failure_points(schedule) -> Vec<FailurePoint>`
  - Scoring factors per task:
    - Position in schedule (later = higher risk, fatigue accumulation)
    - Stress index at that point in the schedule
    - Emotional weight of the task
    - Deadline proximity
    - Duration relative to remaining energy (long tasks late in the day)
  - Output: ranked list of `FailurePoint` structs with `task_id`, `reason`, `risk_score`
- [ ] `src/components/failure_markers.rs` — red warning icons on timeline at predicted failure points
  - Hover tooltip: "High collapse risk — long task after 3 hours of high-stress work"
- [ ] Wire into `ScheduleMetrics.failure_points`

### 8.4 Task Emotional Weighting

Some tasks are psychologically heavy even if they're short. The scheduler should know this.

- [ ] Add `emotional_weight: f64` field to `Task` struct (already added to types)
- [ ] `src/components/emotional_slider.rs` — optional "How heavy does this feel?" slider (0.0–1.0) in task form
  - Default: 0.0 (neutral). User can adjust if a task feels heavier than its duration suggests
  - Preset labels: "Light" (0.0–0.3), "Moderate" (0.3–0.6), "Heavy" (0.6–1.0)
- [ ] Modify Serenity density score: `score = (priority + emotional_weight × 2) / duration`
  - Emotionally heavy tasks get a slight boost so they're tackled when energy is high (early in schedule)
- [ ] Modify Crunch score: `crunch = priority × (1 + urgency) × (1 + emotional_weight × 0.5)`
  - Under pressure, emotionally heavy tasks get weighted more to prevent leaving them until fatigue sets in
- [ ] Stress index formula update: `stress = Σ(pos_w × (pri_w + emotional_weight) × d) / (W × max_p)`

### 8.5 Momentum Engineering

Starts the day with tasks that build confidence, then gradually ramps into harder work.

- [ ] `src/server/momentum.rs` — `apply_momentum_ordering(tasks: &mut [ScheduledTask])`
  - First 2 tasks: sort by `(priority × emotional_weight⁻¹) / duration` — high priority, low emotional weight, short duration = confidence builders
  - Middle tasks: original ordering (density or EDF)
  - Last tasks: can include higher emotional weight (user is warmed up)
  - Compute `momentum_score`: how well the schedule follows the confidence-building curve
    - `momentum = 1.0 - (avg_emotional_weight_first_3_tasks / avg_emotional_weight_all_tasks)`
    - Higher = better momentum (easier tasks up front)
- [ ] Wire into `ScheduleMetrics.momentum_score`
- [ ] `src/components/momentum_indicator.rs` — green/amber/red gauge on analytics dashboard
  - Green: good momentum (easy start, gradual ramp)
  - Red: bad momentum (heavy task first, no warmup)

### 8.6 Decision Debt Tracking

Tracks how many unresolved choices the user is carrying and prompts them to clear mental clutter.

- [ ] `src/server/debt.rs` — `compute_decision_debt(tasks) -> u32`
  - Count tasks that have:
    - No deadline set (open-ended, unresolved)
    - Priority of 3 (medium/undecided)
    - No tags (uncategorised)
  - `decision_debt = count_undeadlined + count_medium_priority + count_untagged`
- [ ] Wire into `ScheduleMetrics.decision_debt`
- [ ] `src/components/debt_counter.rs` — badge in navbar showing decision debt count
  - Click to expand: "You have 7 unresolved decisions. Set deadlines or commit to priorities."
  - Each item links to the task in the form for quick editing
- [ ] Warning threshold: if `decision_debt > 10`, show persistent banner "Your decision debt is high. Clear some choices before generating."

### 8.7 Micro-Recovery Insertion

Automatically places short reset breaks after intense work blocks to reduce fatigue.

- [ ] `src/server/recovery.rs` — `insert_micro_recoveries(schedule: &mut Schedule, config: RecoveryConfig)`
  - After every task with `emotional_weight > 0.6` or `duration > 60`, insert a 5-minute break
  - After every 2 hours of continuous work, insert a 10-minute break
  - Breaks are `Task` objects with `name: "Break"`, `duration: 5 or 10`, `priority: 0`, `emotional_weight: 0.0`
  - Recompute metrics after insertion (breaks reduce stress index, reduce time utilisation)
- [ ] `RecoveryConfig`: configurable break durations, thresholds
- [ ] Timeline: break blocks rendered in light blue with coffee icon
- [ ] Toggle on config panel: "Insert recovery breaks" checkbox (default: on)
- [ ] Adjust `available_time` awareness: breaks consume time budget, so total work time = W - breaks

### 8.8 Identity Conflict Alerts

Warns when the user is trying to act like too many people at once.

- [ ] Add `category: Option<String>` field to `Task` struct (already added to types)
  - Suggested categories: "student", "developer", "applicant", "personal", "health"
  - Dropdown in task form (optional, defaults to None)
- [ ] `src/server/identity.rs` — `detect_identity_conflicts(tasks) -> Vec<String>`
  - If schedule contains tasks from ≥3 distinct categories, flag: "You're wearing too many hats today"
  - If schedule contains both high-priority "applicant" and high-priority "developer" tasks, flag: "Conflicting identities — applicant tasks and developer tasks are competing for your best energy"
- [ ] Wire into `ScheduleMetrics.identity_conflicts`
- [ ] `src/components/identity_alerts.rs` — warning cards on schedule page
  - Each conflict: description + suggested action ("Consider splitting these across two days")

### 8.9 What-If Simulation

Lets the user remove or delay a task and instantly see the impact on stress, deadlines, and workload.

- [ ] `src/components/what_if_panel.rs` — "What if I drop this?" button on each scheduled task
  - Click → runs engine in simulation mode: removes task, regenerates schedule, shows diff
  - Diff display: before/after comparison of all four core metrics + overload flag
  - "What if I delay this?" — move task to end of schedule, show impact
- [ ] `#[server] async fn simulate_change(original_schedule, change: ScheduleChange) -> Result<(Schedule, MetricsDiff), ServerFnError>`
  - `ScheduleChange`: `RemoveTask(String)` or `DelayTask(String)`
  - Returns new schedule + diff struct
- [ ] `src/components/diff_view.rs` — side-by-side or inline diff
  - Green: metric improved (stress down, utilisation down)
  - Red: metric worsened (deadline risk up, productivity down)
  - Summary sentence: "Dropping Task X would reduce your stress by 15% but increase deadline risk by 8%"

### 8.10 Habit Drift Detection

Notices when the user's real behaviour changes over time and updates recommendations before the old model becomes outdated.

- [ ] Extend Convex schema: add `behaviour_log` table
  ```typescript
  behaviour_log: defineTable({
    sessionId: v.string(),
    timestamp: v.string(),
    tasks_planned: v.number(),
    tasks_completed: v.number(),     // user manually marks completion
    avg_priority_selected: v.number(),
    mode_used: v.string(),
    emotional_weight_avg: v.number(),
  })
  ```
- [ ] `src/server/habit.rs` — `detect_habit_drift(session_id) -> Option<DriftAlert>`
  - Fetch last 30 behaviour log entries
  - Compare recent 10 vs earlier 20:
    - If avg priority shifted > 1.0: "Your priorities are changing"
    - If mode usage shifted (was Serenity, now Crunch): "You're under more pressure lately"
    - If completion rate dropped > 20%: "You're completing less than before — overloaded?"
    - If emotional weight trend increasing: "Tasks are feeling heavier over time"
- [ ] `src/components/habit_alert.rs` — notification card on Home page
  - "Your habits have drifted: you've switched to Crunch Mode 80% of the time this week (was 20% last month). Are you okay?"
- [ ] Server function: `#[server] async fn log_behaviour(entry: BehaviourEntry) -> Result<(), ServerFnError>`
  - Called after each schedule generation
- [ ] Server function: `#[server] async fn check_habit_drift(session_id: String) -> Result<Option<DriftAlert>, ServerFnError>`

### 8.11 Phase 8 File Additions

```
src/server/
├── regret.rs                       # Regret Minimisation Mode
├── forecast.rs                     # Failure Forecasting
├── momentum.rs                     # Momentum Engineering
├── debt.rs                         # Decision Debt Tracking
├── recovery.rs                     # Micro-Recovery Insertion
├── identity.rs                     # Identity Conflict Alerts
├── habit.rs                        # Habit Drift Detection

src/components/
├── regret_panel.rs                 # Regret-risk sidebar
├── overload_banner.rs              # Invisible Overload warning
├── failure_markers.rs              # Timeline failure point icons
├── emotional_slider.rs             # Emotional weight input
├── momentum_indicator.rs           # Momentum gauge
├── debt_counter.rs                 # Decision debt badge
├── identity_alerts.rs              # Identity conflict cards
├── what_if_panel.rs                # What-If simulation controls
├── diff_view.rs                    # Before/after metric diff
├── habit_alert.rs                  # Habit drift notification
```

### 8.12 Feature Interaction Matrix

| Feature | Extends Engine | Extends Metrics | Extends UI | Extends Storage |
|---------|---------------|----------------|-----------|----------------|
| Regret Minimisation | Selection | — | regret_panel | — |
| Invisible Overload | — | overload_flag | overload_banner | — |
| Failure Forecasting | — | failure_points | failure_markers | — |
| Emotional Weighting | Scoring (all modes) | stress_index | emotional_slider | — |
| Momentum Engineering | Ordering | momentum_score | momentum_indicator | — |
| Decision Debt | — | decision_debt | debt_counter | — |
| Micro-Recovery | Timeline generation | stress_index, utilisation | timeline (break blocks) | — |
| Identity Conflict | — | identity_conflicts | identity_alerts | category field |
| What-If Simulation | Re-runs engine | All (diff) | what_if_panel, diff_view | — |
| Habit Drift | — | habit_drift_alert | habit_alert | behaviour_log table |
| Push Notifications | — | — | notification_settings | push_subscriptions, scheduled_notifications |

### 8.13 Push Notifications

> **Goal:** Deliver timely, contextual push notifications so the user doesn't have to keep checking the app. Notifications fire even when the browser tab is closed.
>
> **Full algorithmic specifications:** See [`PHASE8_ALGORITHMS.md`](./PHASE8_ALGORITHMS.md#11-push-notifications) for service worker setup, VAPID key flow, scheduling logic, and notification payloads.

#### Architecture

```
┌──────────────┐  subscribe   ┌──────────────────┐  store    ┌──────────────┐
│  Browser      │ ──────────► │  Server Function  │ ───────► │  Convex DB   │
│  (WASM + SW)  │             │  #[server]        │          │  - push_     │
│               │             │                   │          │    subscriptions│
│  Service      │             │  send_push()      │ ◄─────── │  - scheduled_│
│  Worker       │ ◄────────── │                   │  query   │    notifications│
│  (background) │  push event │  web-push crate   │          │              │
└──────────────┘             └──────────────────┘          └──────────────┘
```

- **Service Worker** (`public/sw.js`): registered by WASM client on first load, handles push events and notification clicks
- **VAPID keys**: server authenticates with browser push services (FCM, Mozilla, Apple)
- **Convex scheduled function**: queries `scheduled_notifications` table, sends push when `trigger_at <= now()`
- **web-push crate** (server): encrypts and sends push payloads to subscription endpoints

#### 8.13.1 Service Worker

- [ ] `public/sw.js` — vanilla JS, no WASM
  - `install` event: skip waiting, activate immediately
  - `activate` event: claim clients
  - `push` event: parse payload, show `Notification` with title/body/icon/actions
  - `notificationclick` event: focus or open app tab, navigate to relevant page
  - `pushsubscriptionchange` event: re-subscribe, send new endpoint to server

```javascript
// public/sw.js
self.addEventListener('push', (event) => {
    const data = event.data.json();
    event.waitUntil(
        self.registration.showNotification(data.title, {
            body: data.body,
            icon: '/assets/favicon.ico',
            badge: '/assets/favicon.ico',
            data: { url: data.url, task_id: data.task_id },
            actions: data.actions || [],
            tag: data.tag || 'agamotto',
            renotify: true,
        })
    );
});

self.addEventListener('notificationclick', (event) => {
    event.notification.close();
    const url = event.notification.data.url || '/';
    event.waitUntil(
        clients.matchAll({ type: 'window' }).then((windowClients) => {
            for (const client of windowClients) {
                if (client.url === url && 'focus' in client) {
                    return client.focus();
                }
            }
            return clients.openWindow(url);
        })
    );
});
```

#### 8.13.2 Client-Side Subscription

- [ ] `src/components/notification_prompt.rs` — permission request UI
  - On first visit (or after schedule generation): banner "Enable notifications to get reminders about your schedule"
  - "Enable" button → calls `subscribe_to_push()` server function to get VAPID public key → uses `web-sys` Push API to subscribe → sends subscription to server
  - "Not now" → dismissed, stored in `localStorage`, don't ask again for 7 days
  - "Never" → stored in `localStorage`, never ask again
- [ ] `src/server/push.rs` — `#[server] async fn get_vapid_public_key() -> Result<String, ServerFnError>`
  - Returns the VAPID public key from server env vars
- [ ] `src/server/push.rs` — `#[server] async fn subscribe_to_push(subscription: PushSubscription) -> Result<(), ServerFnError>`
  - Stores subscription endpoint + keys in Convex `push_subscriptions` table
- [ ] `src/server/push.rs` — `#[server] async fn unsubscribe_from_push(endpoint: String) -> Result<(), ServerFnError>`
  - Removes subscription from Convex

```rust
// Client-side subscription using web-sys
async fn subscribe_to_notifications(vapid_key: &str) -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    let navigator = window.navigator();
    let registration = wasm_bindgen_futures::JsFuture::from(
        navigator.service_worker().ready()
    ).await?;

    let sw_registration: ServiceWorkerRegistration = registration.into();
    let push_manager = sw_registration.push_manager();

    let options = PushSubscriptionOptionsInit::new();
    options.set_user_visible_only(true);
    options.set_application_server_key(
        &Uint8Array::from(&base64_url_decode(vapid_key)[..])
    );

    let subscription = wasm_bindgen_futures::JsFuture::from(
        push_manager.subscribe(&options)
    ).await?;

    // Send subscription to server
    let sub: PushSubscription = subscription.into();
    subscribe_to_push(serde_wasm_bindgen::from_value(sub.into())?).await?;

    Ok(())
}
```

#### 8.13.3 Notification Types

| Type | Trigger | Payload | Timing |
|------|---------|---------|--------|
| **Task Starting Soon** | 10 min before scheduled start | "Starting soon: {task_name} ({duration}min, priority {priority})" | 10 min before |
| **Break Over** | At break end time | "Break's over — next up: {next_task_name}" | At break end |
| **Daily Summary** | User's preferred morning time | "You have {n} tasks today. First up: {task_name} at {time}" | Once per day |
| **Overload Warning** | After schedule generation | "Heads up: today's schedule might be too heavy. Consider dropping a task." | Immediate |
| **Habit Drift** | When drift detected on Home load | "Your habits have shifted: {drift_message}" | On detection |
| **Deadline Alert** | 1 hour before any task deadline | "Deadline in 1 hour: {task_name}" | 1 hour before |

#### 8.13.4 Notification Scheduling

- [ ] `src/server/push.rs` — `#[server] async fn schedule_notifications(schedule: Schedule, session_id: String) -> Result<(), ServerFnError>`
  - After schedule generation (or load), creates `scheduled_notifications` entries:
    - For each task: one notification 10 min before `start_time`
    - For each break: one notification at break `end_time` with next task name
    - One daily summary notification at user's preferred time (default: 8:00 AM)
    - Immediate overload warning if `overload_flag == true`
    - Immediate habit drift notification if `check_habit_drift()` returns alert

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
struct ScheduledNotification {
    session_id: String,
    notification_type: String,   // "task_starting", "break_over", "daily_summary", etc.
    trigger_at: String,          // ISO 8601 timestamp
    payload: NotificationPayload,
    sent: bool,                  // flipped to true after push is sent
    subscription_endpoint: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct NotificationPayload {
    title: String,
    body: String,
    url: Option<String>,         // deep link to schedule page
    task_id: Option<String>,     // for click-to-scroll
    tag: String,                 // deduplication tag
    actions: Vec<NotificationAction>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct NotificationAction {
    action: String,              // "view", "snooze", "dismiss"
    title: String,
}
```

#### 8.13.5 Server-Side Push Delivery

- [ ] Convex scheduled function (cron): runs every 1 minute
  - Queries `scheduled_notifications` where `trigger_at <= now()` AND `sent == false`
  - For each: calls `send_push()` with the subscription endpoint and payload
  - Marks `sent = true` after successful send
  - Deletes or archives notifications older than 24 hours

```rust
// Server-side push send using web-push crate
async fn send_push(
    subscription: &PushSubscription,
    payload: &NotificationPayload,
    vapid_private_key: &str,
    vapid_subject: &str,
) -> Result<(), PushError> {
    let content = serde_json::to_string(payload)?;

    let mut builder = WebPushBuilder::new(
        &subscription.endpoint,
        &subscription.keys.p256dh,
        &subscription.keys.auth,
    )?;

    builder.set_vapid_signature(
        VapidSignatureBuilder::from_base64(
            vapid_private_key,
            VapidSignatureBuilder::from_pem(vapid_private_key)?,
        )?
        .build()?,
    );

    builder.set_payload(content.as_bytes());
    let response = builder.send().await?;

    match response.status_code() {
        200..=299 => Ok(()),
        410 => {
            // Subscription expired — remove from DB
            remove_subscription(&subscription.endpoint).await?;
            Ok(())
        }
        _ => Err(PushError::HttpError(response.status_code())),
    }
}
```

#### 8.13.6 Notification Settings UI

- [ ] `src/components/notification_settings.rs` — settings panel (accessible from navbar or Home page)
  - Master toggle: Enable/disable all notifications
  - Per-type toggles:
    - Task reminders (on/off)
    - Break reminders (on/off)
    - Daily summary (on/off, time picker)
    - Overload warnings (on/off)
    - Habit drift alerts (on/off)
    - Deadline alerts (on/off)
  - Snooze all: "Pause notifications for 2 hours"
  - Test notification: "Send a test" button
  - Subscription status: "Active" / "Not subscribed" / "Permission denied"
- [ ] Settings stored in Convex `notification_preferences` table per session
- [ ] `src/server/push.rs` — `#[server] async fn update_notification_prefs(prefs: NotificationPrefs) -> Result<(), ServerFnError>`

#### 8.13.7 Edge Cases

- [ ] Permission denied: show "Notifications blocked. Enable in browser settings." with link to instructions
- [ ] Subscription expired (410 from push service): silently re-subscribe on next page load
- [ ] No subscription: schedule generation works fine, just no notifications
- [ ] Multiple tabs: deduplicate by `tag` field (e.g., `task-{id}-10min` ensures one notification per task)
- [ ] Timezone: all `trigger_at` times stored as UTC, converted to user's local time on display
- [ ] Service worker update: `sw.js` versioned, `skipWaiting` on install, clients claimed on activate

#### 8.13.8 Phase 8.13 File Additions

```
public/
├── sw.js                             # Service worker (push events, notification display)

src/server/
├── push.rs                           # Server functions: subscribe, unsubscribe, schedule, send, prefs

src/components/
├── notification_prompt.rs            # Permission request banner
├── notification_settings.rs          # Notification preferences panel
```
| Push Notifications | — | — | notification_settings, permission_prompt | push_subscriptions, scheduled_notifications |

---

## File Structure (Target)

```
src/
├── main.rs                         # App entry, Route enum, App component
├── types.rs                        # Task, Schedule, ScheduledTask, ScheduleMetrics, ScheduleMode
├── error.rs                        # ScheduleError enum
├── state.rs                        # Global Signal types
├── server/
│   ├── mod.rs                      # Server module exports
│   ├── greedy.rs                   # #[server] schedule_serenity()
│   ├── knapsack.rs                 # #[server] schedule_crunch()
│   ├── edf.rs                      # order_by_edf() helper
│   ├── metrics.rs                  # compute_metrics() helper
│   ├── persistence.rs              # #[server] save/load/list schedules
│   ├── convex_client.rs            # Convex SDK wrapper (typed helpers for queries/mutations)
│   ├── regret.rs                   # Phase 8: Regret Minimisation Mode
│   ├── forecast.rs                 # Phase 8: Failure Forecasting
│   ├── momentum.rs                 # Phase 8: Momentum Engineering
│   ├── debt.rs                     # Phase 8: Decision Debt Tracking
│   ├── recovery.rs                 # Phase 8: Micro-Recovery Insertion
│   ├── identity.rs                 # Phase 8: Identity Conflict Alerts
│   ├── habit.rs                    # Phase 8: Habit Drift Detection
│   └── push.rs                     # Phase 8: Push notifications (subscribe, schedule, send)
├── components/
│   ├── mod.rs                      # Component exports
│   ├── navbar.rs                   # Top navigation bar
│   ├── home.rs                     # Home page layout
│   ├── task_form.rs                # Task entry form
│   ├── task_list.rs                # Scrollable task list with edit/delete
│   ├── config_panel.rs             # Time window + mode + generate button
│   ├── schedule_view.rs            # Schedule page layout
│   ├── timeline.rs                 # Horizontal bar chart visualisation
│   ├── metrics_panel.rs            # Four-metric analytics dashboard
│   ├── excluded_list.rs            # Excluded tasks with reasons
│   ├── schedule_list.rs            # Numbered list with drag-and-drop
│   ├── saved_schedules.rs          # Save/load dropdown
│   ├── regret_panel.rs             # Phase 8: Regret-risk sidebar
│   ├── overload_banner.rs          # Phase 8: Invisible Overload warning
│   ├── failure_markers.rs          # Phase 8: Timeline failure point icons
│   ├── emotional_slider.rs         # Phase 8: Emotional weight input
│   ├── momentum_indicator.rs       # Phase 8: Momentum gauge
│   ├── debt_counter.rs             # Phase 8: Decision debt badge
│   ├── identity_alerts.rs          # Phase 8: Identity conflict cards
│   ├── what_if_panel.rs            # Phase 8: What-If simulation controls
│   ├── diff_view.rs                # Phase 8: Before/after metric diff
│   ├── habit_alert.rs              # Phase 8: Habit drift notification
│   ├── notification_prompt.rs      # Phase 8: Permission request banner
│   └── notification_settings.rs    # Phase 8: Notification preferences panel
└── assets/
    ├── main.css                    # Base styles
    ├── tailwind.css                # Tailwind utilities
    └── favicon.ico                 # App icon

public/
├── sw.js                           # Phase 8: Service worker (push events, notification display)

convex/
├── schema.ts                       # Database schema (tasks, schedules, behaviour_log)
└── _generated/                     # Convex auto-generated types
```

---

## Tech Stack

| Layer | Technology | Notes |
|-------|-----------|-------|
| Language | Rust (edition 2021) | Both client (WASM) and server |
| Web Framework | Dioxus 0.7.1 | Fullstack: `web` + `server` features |
| Styling | Tailwind CSS + custom CSS | Responsive breakpoints, utility-first |
| Serialization | Serde + JSON | Server↔client IPC, Convex storage |
| Database | Convex.dev | Serverless, real-time, accessed server-side |
| Convex Client | [`convex`](https://crates.io/crates/convex) v0.10.3 | Official Rust SDK — query, mutation, subscribe |
| Push Notifications | web-push (server) + Service Worker (browser) | VAPID auth, Web Push Protocol |
| Browser APIs | web-sys (WASM) | Push API, Service Worker registration |
| Build | Cargo + dx CLI | `dx serve` for dev, `dx build` for production |

---

## Dependencies

```toml
[package]
name = "agamotto"
version = "0.1.0"
edition = "2021"

[dependencies]
dioxus = { version = "0.7.1", features = ["router", "fullstack"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
convex = "0.10.3"          # Official Convex Rust SDK
dotenvy = "0.15"            # Load .env.local for CONVEX_URL
web-push = "0.10"        # Phase 8: server-side push notification sending
web-sys = { version = "0.3", features = [
    "PushManager",
    "PushSubscription",
    "PushSubscriptionOptionsInit",
    "ServiceWorkerContainer",
    "ServiceWorkerRegistration",
    "Notification",
    "NotificationOptions",
    "NotificationAction",
] }                        # Phase 8: client-side Push API + Service Worker bindings
wasm-bindgen = "0.2"      # Phase 8: JS interop for service worker registration
wasm-bindgen-futures = "0.4" # Phase 8: async JS interop
maplit = "1"               # Convenience macros for BTreeMap (Convex args)

[features]
default = ["web"]
web = ["dioxus/web"]
server = ["dioxus/server"]
```

> Note: `desktop` and `mobile` features removed — this is web-only.

---

## Key Algorithms

| Mode | Selection | Ordering | Server Complexity |
|------|-----------|----------|-------------------|
| Serenity | Greedy by priority/density | Shortest-first (↑ duration) | O(n log n) |
| Crunch | 0/1 Knapsack DP | EDF (↑ deadline) | O(n · W) |
| Regret Minimisation (Phase 8) | Dual-run Serenity + Crunch, compare regret | Minimises max regret | O(n · W) |

---

## Evaluation Metrics

### Core Metrics (Phase 1)

| # | Metric | Formula | Range |
|---|--------|---------|-------|
| 1 | Productivity Score | Σ(pᵢ) / Σ(p\_all) × 100% | 0–100% |
| 2 | Time Utilisation | Σ(dᵢ) / W × 100% | 0–100% |
| 3 | Stress Index | Σ(pos\_wᵢ × pri\_wᵢ × dᵢ) / (W × max\_p) | 0.0–1.0 |
| 4 | Deadline Risk | Σ(riskᵢ) / \|S\| × 100% | 0–100% |

### Extended Metrics (Phase 8)

| # | Metric | Source | Range |
|---|--------|--------|-------|
| 5 | Overload Flag | stress > 0.65 ∧ utilisation > 0.9 ∧ avg\_emotional > 0.5 | bool |
| 6 | Failure Points | Position + stress + emotional + deadline + duration scoring | Vec\<FailurePoint\> |
| 7 | Decision Debt | count(undeadlined) + count(priority=3) + count(untagged) | 0–3n |
| 8 | Momentum Score | 1.0 − (avg\_emotional\_first\_3 / avg\_emotional\_all) | 0.0–1.0 |
| 9 | Identity Conflicts | Distinct categories ≥ 3 or high-priority cross-category clash | Vec\<String\> |
| 10 | Habit Drift Alert | Recent vs historical behaviour divergence > threshold | bool |
| 11 | Regret Score | priority × (1 / hours\_until\_deadline) × (1 + emotional\_weight) | 0.0–∞ |
