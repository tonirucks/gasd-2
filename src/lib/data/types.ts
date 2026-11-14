/** IPC models mirror gasd-core's camelCase serde models. */
export type Section = { id: number; name: string; position: number };
export type TaskStatus = "open" | "inProgress" | "waiting" | "done";
export type Task = {
  id: number;
  title: string;
  sectionId: number;
  status: TaskStatus;
  statusChangedAt: number;
  position: number;
};
export type TaskUpdate = Pick<Task, "title" | "sectionId" | "status">;
export type TaskList = { sectionId: number; status: TaskStatus };
export type ListData = { sections: Section[]; tasks: Task[] };
export type CommandError = { code: string; message: string };
