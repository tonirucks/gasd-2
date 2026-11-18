import type { Task, TaskUpdate } from "$lib/data/types";

export type TaskDraft = {
  id: number;
  title: string;
  description: string;
  savedTitle: string;
  savedDescription: string;
};

/** A save must not overwrite text entered while the request was in flight. */
export async function persistTaskDraft(
  draft: TaskDraft,
  task: Task,
  update: (id: number, update: TaskUpdate) => Promise<boolean>,
): Promise<boolean> {
  const originalTitle = draft.title;
  const title = originalTitle.trim();
  const description = draft.description;
  if (!title) return false;
  if (!await update(task.id, { title, description, sectionId: task.sectionId, status: task.status })) return false;
  if (draft.title === originalTitle) draft.title = title;
  draft.savedTitle = title;
  draft.savedDescription = description;
  return true;
}
