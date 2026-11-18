import { describe, expect, it } from "vitest";
import { destinationOrder, inColumn, isVisible, previewColumn } from "./board";
import type { Task } from "./types";

const now = new Date(2026, 10, 14, 12).getTime();
const midnight = new Date(2026, 10, 14).getTime();
function task(id: number, fields: Partial<Task> = {}): Task {
  return { id, title: `Task ${id}`, description: "", sectionId: 1, status: "open", statusChangedAt: now, position: 0, subtasks: [], ...fields };
}

describe("board visibility", () => {
  it("uses the local calendar day, including midnight but excluding tomorrow", () => {
    expect(isVisible(task(1, { status: "done", statusChangedAt: midnight }), now)).toBe(true);
    expect(isVisible(task(1, { status: "done", statusChangedAt: midnight - 1 }), now)).toBe(false);
    const tomorrow = new Date(2026, 10, 15).getTime();
    expect(isVisible(task(1, { status: "done", statusChangedAt: tomorrow }), now)).toBe(false);
    expect(isVisible(task(1, { status: "waiting", statusChangedAt: midnight - 1 }), now)).toBe(true);
    expect(isVisible(task(1, { status: "done" }), tomorrow)).toBe(false);
  });

  it("handles daylight saving days using calendar boundaries rather than 24-hour offsets", () => {
    for (const [month, day] of [[2, 8], [2, 29], [9, 25], [10, 1]]) {
      const date = new Date(2026, month, day, 12).getTime();
      const start = new Date(2026, month, day).getTime();
      const end = new Date(2026, month, day + 1).getTime();
      expect(isVisible(task(1, { status: "done", statusChangedAt: start }), date)).toBe(true);
      expect(isVisible(task(1, { status: "done", statusChangedAt: end - 1 }), date)).toBe(true);
      expect(isVisible(task(1, { status: "done", statusChangedAt: end }), date)).toBe(false);
    }
  });
});

describe("column dragging", () => {
  const column = { sectionId: 1, status: "done" } as const;
  const old = task(1, { status: "done", statusChangedAt: midnight - 1 });
  const today = task(2, { status: "done" });
  const moving = task(3, { sectionId: 2, status: "waiting" });

  it("previews section/status changes without removing hidden tasks or mutating stored data", () => {
    const stored = [old, today, moving];
    const result = previewColumn(stored, [moving, today], column, now);
    expect(result.map((item) => item.id)).toEqual([1, 3, 2]);
    expect(result[1]).toMatchObject({ sectionId: 1, status: "done", statusChangedAt: now });
    expect(isVisible(result[1], now)).toBe(true);
    expect(moving).toMatchObject({ sectionId: 2, status: "waiting" });
    expect(result[0]).toEqual(old);
    expect(inColumn(moving, column)).toBe(false);
  });

  it("sends hidden membership and visible order, excluding drag placeholders", () => {
    const shadow = task(-1, { isDndShadowItem: true });
    expect(destinationOrder([old, today, moving], [moving, shadow, today], column, now)).toEqual([1, 3, 2]);
    expect(destinationOrder([old, today], [], column, now)).toEqual([1]);
  });

  it("keeps full membership for an empty visible done column", () => {
    expect(destinationOrder([old, moving], [moving], column, now)).toEqual([1, 3]);
  });

  it("does not silently supply missing visible tasks, allowing core to reject stale orders", () => {
    expect(destinationOrder([old, today], [moving], column, now)).toEqual([1, 3]);
  });

  it("preserves other columns and timestamps during reorder previews", () => {
    const open = task(4);
    const result = previewColumn([old, today, open], [today], column, now + 100);
    expect(result).toEqual([old, open, today]);
  });
});
