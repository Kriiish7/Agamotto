import { defineSchema, defineTable } from "convex/server";
import { v } from "convex/values";

export default defineSchema({
  schedules: defineTable({
    sessionId: v.string(),
    name: v.string(),
    mode: v.string(),
    available_time: v.number(),
    tasks_json: v.string(),
    metrics_json: v.string(),
  }),

  behaviour_log: defineTable({
    sessionId: v.string(),
    timestamp: v.string(),
    tasks_planned: v.number(),
    tasks_completed: v.number(),
    avg_priority_selected: v.number(),
    mode_used: v.string(),
    emotional_weight_avg: v.number(),
  }),

  push_subscriptions: defineTable({
    sessionId: v.string(),
    endpoint: v.string(),
    p256dh: v.string(),
    auth: v.string(),
    created_at: v.string(),
  }),

  scheduled_notifications: defineTable({
    sessionId: v.string(),
    notification_type: v.string(),
    trigger_at: v.string(),
    payload_json: v.string(),
    sent: v.boolean(),
    subscription_endpoint: v.string(),
  }),

  notification_preferences: defineTable({
    sessionId: v.string(),
    enabled: v.boolean(),
    task_reminders: v.boolean(),
    break_reminders: v.boolean(),
    daily_summary: v.boolean(),
    daily_summary_time: v.string(),
    overload_warnings: v.boolean(),
    habit_drift_alerts: v.boolean(),
    deadline_alerts: v.boolean(),
    snoozed_until: v.optional(v.string()),
  }),
});
