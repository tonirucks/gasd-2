/** IPC models mirror gasd-core's camelCase serde models. */
export type Section = { id: number; name: string; position: number };
export type TaskStatus = "open" | "inProgress" | "waiting" | "done";
export type Subtask = { id: number; title: string; completed: boolean };
export type Task = {
  id: number;
  title: string;
  description: string;
  sectionId: number;
  status: TaskStatus;
  statusChangedAt: number;
  position: number;
  subtasks: Subtask[];
  jira?: { key: string; url: string };
};
export type TaskUpdate = Pick<Task, "title" | "description" | "sectionId" | "status">;
export type TaskList = { sectionId: number; status: TaskStatus };
export type ListData = { sections: Section[]; tasks: Task[] };
export type CommandError = { code: string; message: string };
export type JiraQuery = { id: number; name: string; jql: string };
export type JiraTicket = { key: string; summary: string; status: string; url: string };
export type JiraPage = { tickets: JiraTicket[]; nextPageToken: string | null };
export type JiraConnectionSettings = { site: string; email: string; hasToken: boolean };
export type AppSettings = { jira: JiraConnectionSettings };
