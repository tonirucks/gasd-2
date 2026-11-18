import { describe, expect, it, vi } from "vitest";
import { persistTaskDraft, type TaskDraft } from "./task-draft";
import type { Task } from "$lib/data/types";

const task: Task = {
  id: 1, title: "Review", description: "", sectionId: 2, status: "waiting",
  statusChangedAt: 1, position: 0, subtasks: [],
};

function draft(): TaskDraft {
  return { id: 1, title: "  Review changes  ", description: "New details", savedTitle: "Review", savedDescription: "" };
}

describe("task autosave", () => {
  it("saves title and description while preserving the current section and status", async () => {
    const value = draft();
    const update = vi.fn().mockResolvedValue(true);
    expect(await persistTaskDraft(value, task, update)).toBe(true);
    expect(update).toHaveBeenCalledWith(1, {
      title: "Review changes", description: "New details", sectionId: 2, status: "waiting",
    });
    expect(value).toMatchObject({
      title: "Review changes", savedTitle: "Review changes", savedDescription: "New details",
    });
  });

  it("keeps newer edits made during a save for the next autosave", async () => {
    const value = draft();
    let finish!: (saved: boolean) => void;
    const update = vi.fn(() => new Promise<boolean>((resolve) => { finish = resolve; }));
    const saving = persistTaskDraft(value, task, update);
    value.title = "Keep typing";
    value.description = "More details";
    finish(true);
    expect(await saving).toBe(true);
    expect(value).toMatchObject({
      title: "Keep typing", description: "More details",
      savedTitle: "Review changes", savedDescription: "New details",
    });
  });

  it("retains the draft and its last saved values after a failure, allowing retry", async () => {
    const value = draft();
    const original = { ...value };
    const update = vi.fn().mockResolvedValueOnce(false).mockResolvedValueOnce(true);
    expect(await persistTaskDraft(value, task, update)).toBe(false);
    expect(value).toEqual(original);
    expect(await persistTaskDraft(value, task, update)).toBe(true);
    expect(value.savedDescription).toBe("New details");
  });

  it("does not send an empty title", async () => {
    const value = { ...draft(), title: " \n " };
    const update = vi.fn();
    expect(await persistTaskDraft(value, task, update)).toBe(false);
    expect(update).not.toHaveBeenCalled();
  });

  it("can clear the description", async () => {
    const value = { ...draft(), description: "", savedDescription: "Old description" };
    const update = vi.fn().mockResolvedValue(true);
    expect(await persistTaskDraft(value, task, update)).toBe(true);
    expect(value.savedDescription).toBe("");
    expect(update.mock.calls[0][1].description).toBe("");
  });
});
