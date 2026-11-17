import { invoke as invokeNative, isTauri } from "@tauri-apps/api/core";
import type { AppSettings, JiraPage, JiraQuery, ListData, Section, Task, TaskList, TaskUpdate } from "./types";

function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) {
    return Promise.reject(new Error("Open the desktop app to access your list. Native data is unavailable in the browser."));
  }
  return invokeNative<T>(command, args);
}

/** All native communication lives here; components never invoke commands directly. */
export const listClient = {
  get: () => invoke<ListData>("list_get"),
  sections: {
    create: (name: string) => invoke<Section>("sections_create", { name }),
    rename: (id: number, name: string) => invoke<Section>("sections_rename", { id, name }),
    delete: (id: number) => invoke<void>("sections_delete", { id }),
    reorder: (ids: number[]) => invoke<void>("sections_reorder", { ids }),
  },
  tasks: {
    create: (sectionId: number, title: string) => invoke<Task>("tasks_create", { sectionId, title }),
    update: (id: number, update: TaskUpdate) => invoke<Task>("tasks_update", { id, update }),
    delete: (id: number) => invoke<void>("tasks_delete", { id }),
    reorder: (list: TaskList, ids: number[]) => invoke<void>("tasks_reorder", { list, ids }),
    place: (id: number, list: TaskList, ids: number[]) => invoke<void>("tasks_place", { id, list, ids }),
    linkJira: (taskId: number, key: string | null) => invoke<Task>("jira_task_link", { taskId, key }),
  },
};

export type ListClient = typeof listClient;

export const jiraClient = {
  queries: () => invoke<JiraQuery[]>("jira_queries"),
  saveQuery: (id: number | null, name: string, jql: string) => invoke<JiraQuery>("jira_query_save", { id, name, jql }),
  deleteQuery: (id: number) => invoke<void>("jira_query_delete", { id }),
  search: (queryId: number, nextPageToken: string | null = null) => invoke<JiraPage>("jira_search", { queryId, nextPageToken }),
};

export const settingsClient = {
  get: () => invoke<AppSettings>("settings_get"),
  saveJira: (site: string, email: string, token: string | null) => invoke<AppSettings>("settings_save_jira", { site, email, token }),
  clearJira: () => invoke<void>("settings_clear_jira"),
  testJira: () => invoke<void>("jira_test_connection"),
};

export type SettingsClient = typeof settingsClient;
