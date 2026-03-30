import { mutation, query } from "./_generated/server";
import { v } from "convex/values";

export const save = mutation({
  args: {
    sessionId: v.string(),
    name: v.string(),
    mode: v.string(),
    available_time: v.number(),
    tasks_json: v.string(),
    metrics_json: v.string(),
  },
  handler: async (ctx, args) => {
    const id = await ctx.db.insert("schedules", args);
    return id;
  },
});

export const load = query({
  args: { id: v.id("schedules") },
  handler: async (ctx, args) => {
    const doc = await ctx.db.get(args.id);
    if (!doc) throw new Error("Schedule not found");
    return doc;
  },
});

export const list = query({
  args: { sessionId: v.string() },
  handler: async (ctx, args) => {
    const docs = await ctx.db
      .query("schedules")
      .filter((q) => q.eq(q.field("sessionId"), args.sessionId))
      .order("desc")
      .take(50);

    return docs.map((doc) => ({
      id: doc._id,
      name: doc.name,
      mode: doc.mode,
      available_time: doc.available_time,
    }));
  },
});

export const remove = mutation({
  args: { id: v.id("schedules") },
  handler: async (ctx, args) => {
    await ctx.db.delete(args.id);
  },
});
