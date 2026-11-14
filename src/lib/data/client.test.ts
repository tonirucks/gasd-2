import { beforeEach, describe, expect, it, vi } from "vitest";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { listClient } from "./client";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn(), isTauri: vi.fn() }));

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(isTauri).mockReturnValue(true);
});

describe("native client", () => {
  it("uses domain command names and camelCase arguments", async () => {
    await listClient.get();
    await listClient.sections.create("Work");
    await listClient.sections.rename(1, "Home");
    await listClient.sections.reorder([2, 1]);
    await listClient.sections.delete(1);
    await listClient.tasks.create(2, "Read");
    await listClient.tasks.update(3, { title: "Read more", sectionId: 2, done: true });
    await listClient.tasks.reorder({ kind: "done" }, [3]);
    await listClient.tasks.place(3, { kind: "active", sectionId: 2 }, [4, 3]);
    await listClient.tasks.delete(3);
    expect(vi.mocked(invoke).mock.calls).toEqual([
      ["list_get", undefined],
      ["sections_create", { name: "Work" }],
      ["sections_rename", { id: 1, name: "Home" }],
      ["sections_reorder", { ids: [2, 1] }],
      ["sections_delete", { id: 1 }],
      ["tasks_create", { sectionId: 2, title: "Read" }],
      ["tasks_update", { id: 3, update: { title: "Read more", sectionId: 2, done: true } }],
      ["tasks_reorder", { list: { kind: "done" }, ids: [3] }],
      ["tasks_place", { id: 3, list: { kind: "active", sectionId: 2 }, ids: [4, 3] }],
      ["tasks_delete", { id: 3 }],
    ]);
  });

  it("explains why data is unavailable outside Tauri", async () => {
    vi.mocked(isTauri).mockReturnValue(false);
    await expect(listClient.get()).rejects.toThrow("Open the desktop app");
    expect(invoke).not.toHaveBeenCalled();
  });
});
