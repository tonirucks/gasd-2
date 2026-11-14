import type { TaskList, TaskStatus } from "$lib/data/types";
import type { Task } from "./types";

export const columns: { status: TaskStatus; label: string }[] = [
  { status: "open", label: "Open" },
  { status: "inProgress", label: "In progress" },
  { status: "waiting", label: "Waiting" },
  { status: "done", label: "Done" },
];


export function isVisible(task: Task, now: number): boolean {
  if (task.status !== "done") return true;
  const date = new Date(now);
  const start = new Date(date.getFullYear(), date.getMonth(), date.getDate()).getTime();
  const end = new Date(date.getFullYear(), date.getMonth(), date.getDate() + 1).getTime();
  return task.statusChangedAt >= start && task.statusChangedAt < end;
}

export function inColumn(task: Task, column: TaskList): boolean {
  return task.sectionId === column.sectionId && task.status === column.status;
}

/** Replace a visible drag zone without dropping hidden completed tasks. */
export function previewColumn(tasks: Task[], incoming: Task[], column: TaskList, now: number): Task[] {
  const incomingIds = new Set(incoming.map((task) => task.id));
  return [
    ...tasks.filter((task) => !(inColumn(task, column) && isVisible(task, now)) && !incomingIds.has(task.id)),
    ...incoming.map((task): Task => ({
      ...task,
      sectionId: column.sectionId,
      status: column.status,
      statusChangedAt: task.status === column.status ? task.statusChangedAt : now,
    })),
  ];
}

/** Core validates the entire column, including older completed tasks hidden by the UI. */
export function destinationOrder(stored: Task[], incoming: Task[], column: TaskList, now: number): number[] {
  const visible = incoming.filter((task) => !task.isDndShadowItem).map((task) => task.id);
  const supplied = new Set(visible);
  const hidden = stored.filter((task) => inColumn(task, column) && !isVisible(task, now) && !supplied.has(task.id));
  return [...hidden.map((task) => task.id), ...visible];
}
