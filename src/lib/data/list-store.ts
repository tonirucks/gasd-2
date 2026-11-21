import { get, writable } from "svelte/store";
import { listClient, type ListClient } from "./client";
import type { ListData, TaskUpdate } from "./types";

type ListState = {
  data: ListData;
  loaded: boolean;
  busy: boolean;
  error: string | null;
};

function errorMessage(error: unknown): string {
  if (typeof error === "object" && error !== null && "message" in error) {
    return String(error.message);
  }
  return typeof error === "string" ? error : "Could not access your list. Please try again.";
}

/** One store per page. SQLite is authoritative; DnD previews stay in the view. */
export function createListStore(client: ListClient = listClient) {
  const state = writable<ListState>({
    data: { sections: [], tasks: [] }, loaded: false, busy: false, error: null,
  });

  async function refresh(): Promise<void> {
    const data = await client.get();
    state.update((value) => ({ ...value, data, loaded: true }));
  }

  async function run(operation?: () => Promise<unknown>): Promise<boolean> {
    if (get(state).busy) return false;
    state.update((value) => ({ ...value, busy: true, error: null }));
    let saved = false;
    try {
      if (operation) {
        await operation();
        saved = true;
      }
      await refresh();
      return true;
    } catch (error) {
      // A stale drag order or missing item may come from concurrent CLI edits.
      // Reload on failure rather than leaving optimistic previews on screen.
      if (operation) {
        try { await refresh(); } catch { /* Keep the last known data. */ }
      }
      state.update((value) => ({
        ...value,
        data: { ...value.data },
        error: saved
          ? `Saved, but could not refresh your list: ${errorMessage(error)}`
          : errorMessage(error),
      }));
      return saved;
    } finally {
      state.update((value) => ({ ...value, busy: false }));
    }
  }

  return {
    subscribe: state.subscribe,
    load: () => run(),
    createSection: (name: string) => run(() => client.sections.create(name)),
    renameSection: (id: number, name: string) => run(() => client.sections.rename(id, name)),
    deleteSection: (id: number) => run(() => client.sections.delete(id)),
    reorderSections: (ids: number[]) => run(() => client.sections.reorder(ids)),
    createTask: (sectionId: number, title: string) => run(() => client.tasks.create(sectionId, title)),
    updateTask: (id: number, update: TaskUpdate) =>
      run(() => client.tasks.update(id, update)),
    deleteTask: (id: number) => run(() => client.tasks.delete(id)),
    linkJira: (id: number, key: string | null) => run(() => client.tasks.linkJira(id, key)),
    linkFolder: (id: number, folder: string | null) => run(() => client.tasks.linkFolder(id, folder)),
    createSubtask: (...args: Parameters<ListClient["subtasks"]["create"]>) => run(() => client.subtasks.create(...args)),
    updateSubtask: (...args: Parameters<ListClient["subtasks"]["update"]>) => run(() => client.subtasks.update(...args)),
    deleteSubtask: (...args: Parameters<ListClient["subtasks"]["delete"]>) => run(() => client.subtasks.delete(...args)),
    placeTask: (...args: Parameters<ListClient["tasks"]["place"]>) => run(() => client.tasks.place(...args)),
  };
}
