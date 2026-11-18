import { get } from "svelte/store";
import { describe, expect, it, vi } from "vitest";
import { createListStore } from "./list-store";
import type { ListClient } from "./client";
import type { ListData } from "./types";

function setup() {
  const data: ListData = {
    sections: [{ id: 1, name: "Work", position: 0 }],
    tasks: [{ id: 2, title: "Review", description: "", sectionId: 1, status: "open", statusChangedAt: 1, position: 0 }],
  };
  const client = {
    get: vi.fn().mockResolvedValue(data),
    sections: { create: vi.fn(), rename: vi.fn(), delete: vi.fn(), reorder: vi.fn() },
    tasks: { create: vi.fn(), update: vi.fn(), delete: vi.fn(), reorder: vi.fn(), place: vi.fn(), linkJira: vi.fn() },
  } satisfies ListClient;
  return { data, client, store: createListStore(client) };
}

describe("list store", () => {
  it("saves and clears task descriptions and refreshes canonical data", async () => {
    const { store, client, data } = setup();
    const update = { title: "Review", description: "Check the changes\nThen approve", sectionId: 1, status: "open" } as const;
    client.get.mockResolvedValueOnce({ ...data, tasks: [{ ...data.tasks[0], ...update }] });
    expect(await store.updateTask(2, update)).toBe(true);
    expect(client.tasks.update).toHaveBeenLastCalledWith(2, update);
    expect(get(store).data.tasks[0].description).toBe(update.description);
    expect(await store.updateTask(2, { ...update, description: "" })).toBe(true);
    expect(client.tasks.update).toHaveBeenLastCalledWith(2, { ...update, description: "" });
    expect(get(store).data.tasks[0].description).toBe("");
  });

  it("refreshes after linking and unlinking Jira tickets", async () => {
    const { store, client } = setup();
    expect(await store.linkJira(2, "TEAM-1")).toBe(true);
    expect(client.tasks.linkJira).toHaveBeenLastCalledWith(2, "TEAM-1");
    expect(await store.linkJira(2, null)).toBe(true);
    expect(client.tasks.linkJira).toHaveBeenLastCalledWith(2, null);
    expect(client.get).toHaveBeenCalledTimes(2);
  });
  it("loads canonical data and refreshes after a mutation", async () => {
    const { store, client, data } = setup();
    expect(await store.load()).toBe(true);
    expect(get(store)).toMatchObject({ loaded: true, busy: false, error: null, data });
    expect(await store.createTask(1, "Next")).toBe(true);
    expect(client.tasks.create).toHaveBeenCalledWith(1, "Next");
    expect(client.get).toHaveBeenCalledTimes(2);
  });

  it("recovers from initial load failure and supports retry", async () => {
    const { store, client } = setup();
    client.get.mockRejectedValueOnce(new Error("Offline"));
    expect(await store.load()).toBe(false);
    expect(get(store)).toMatchObject({ loaded: false, busy: false, error: "Offline" });
    expect(await store.load()).toBe(true);
    expect(get(store).error).toBeNull();
  });

  it("reloads concurrent CLI changes after a rejected drag", async () => {
    const { store, client, data } = setup();
    await store.load();
    client.tasks.place.mockRejectedValueOnce({ code: "invalidOrder", message: "Stale order" });
    const changed = { ...data, tasks: [] };
    client.get.mockResolvedValueOnce(changed);
    expect(await store.placeTask(2, { sectionId: 1, status: "done" }, [2])).toBe(false);
    expect(get(store)).toMatchObject({ data: changed, error: "Stale order", busy: false });
  });

  it("keeps known data when both a mutation and recovery read fail", async () => {
    const { store, client, data } = setup();
    await store.load();
    client.tasks.delete.mockRejectedValueOnce("Database locked");
    client.get.mockRejectedValueOnce("Unavailable");
    expect(await store.deleteTask(2)).toBe(false);
    expect(get(store)).toMatchObject({ data, error: "Database locked", busy: false });
  });

  it("does not duplicate a saved mutation when refreshing fails", async () => {
    const { store, client } = setup();
    await store.load();
    client.get.mockRejectedValueOnce("Unavailable");
    expect(await store.createSection("Home")).toBe(true);
    expect(get(store).error).toContain("Saved, but could not refresh");
    await store.load();
    expect(client.sections.create).toHaveBeenCalledTimes(1);
  });

  it("prevents overlapping writes", async () => {
    const { store, client } = setup();
    let finish!: () => void;
    client.tasks.delete.mockImplementationOnce(() => new Promise<void>((resolve) => { finish = resolve; }));
    const pending = store.deleteTask(2);
    expect(get(store).busy).toBe(true);
    expect(await store.createSection("Ignored")).toBe(false);
    expect(client.sections.create).not.toHaveBeenCalled();
    finish();
    expect(await pending).toBe(true);
    expect(get(store).busy).toBe(false);
  });
});
