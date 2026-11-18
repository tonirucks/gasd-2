import { beforeEach, describe, expect, it, vi } from "vitest";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { jiraClient, listClient, settingsClient } from "./client";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn(), isTauri: vi.fn() }));

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(isTauri).mockReturnValue(true);
});

describe("native client", () => {
  it("uses settings commands and can preserve an existing token", async () => {
    await settingsClient.get();
    await settingsClient.saveJira("https://team.atlassian.net", "me@example.com", "new-token");
    await settingsClient.saveJira("https://team.atlassian.net", "me@example.com", null);
    await settingsClient.testJira();
    await settingsClient.clearJira();
    expect(vi.mocked(invoke).mock.calls).toEqual([
      ["settings_get", undefined],
      ["settings_save_jira", { site: "https://team.atlassian.net", email: "me@example.com", token: "new-token" }],
      ["settings_save_jira", { site: "https://team.atlassian.net", email: "me@example.com", token: null }],
      ["jira_test_connection", undefined],
      ["settings_clear_jira", undefined],
    ]);
  });
  it("uses domain command names and camelCase arguments", async () => {
    await listClient.get();
    await listClient.sections.create("Work");
    await listClient.sections.rename(1, "Home");
    await listClient.sections.reorder([2, 1]);
    await listClient.sections.delete(1);
    await listClient.tasks.create(2, "Read");
    await listClient.tasks.update(3, { title: "Read more", description: "Chapter one\nChapter two", sectionId: 2, status: "done" });
    await listClient.tasks.reorder({ sectionId: 2, status: "done" }, [3]);
    await listClient.tasks.place(3, { sectionId: 2, status: "inProgress" }, [4, 3]);
    await listClient.tasks.delete(3);
    expect(vi.mocked(invoke).mock.calls).toEqual([
      ["list_get", undefined],
      ["sections_create", { name: "Work" }],
      ["sections_rename", { id: 1, name: "Home" }],
      ["sections_reorder", { ids: [2, 1] }],
      ["sections_delete", { id: 1 }],
      ["tasks_create", { sectionId: 2, title: "Read" }],
      ["tasks_update", { id: 3, update: { title: "Read more", description: "Chapter one\nChapter two", sectionId: 2, status: "done" } }],
      ["tasks_reorder", { list: { sectionId: 2, status: "done" }, ids: [3] }],
      ["tasks_place", { id: 3, list: { sectionId: 2, status: "inProgress" }, ids: [4, 3] }],
      ["tasks_delete", { id: 3 }],
    ]);
  });

  it("explains why data is unavailable outside Tauri", async () => {
    vi.mocked(isTauri).mockReturnValue(false);
    await expect(listClient.get()).rejects.toThrow("Open the desktop app");
    expect(invoke).not.toHaveBeenCalled();
  });

  it("uses typed Jira commands including pagination and unlinking", async () => {
    await jiraClient.queries();
    await jiraClient.saveQuery(null, "Mine", "assignee = currentUser()");
    await jiraClient.saveQuery(1, "Team", "project = TEAM");
    await jiraClient.search(1);
    await jiraClient.search(1, "page-2");
    await jiraClient.deleteQuery(1);
    await listClient.tasks.linkJira(2, "TEAM-1");
    await listClient.tasks.linkJira(2, null);
    expect(vi.mocked(invoke).mock.calls).toEqual([
      ["jira_queries", undefined],
      ["jira_query_save", { id: null, name: "Mine", jql: "assignee = currentUser()" }],
      ["jira_query_save", { id: 1, name: "Team", jql: "project = TEAM" }],
      ["jira_search", { queryId: 1, nextPageToken: null }],
      ["jira_search", { queryId: 1, nextPageToken: "page-2" }],
      ["jira_query_delete", { id: 1 }],
      ["jira_task_link", { taskId: 2, key: "TEAM-1" }],
      ["jira_task_link", { taskId: 2, key: null }],
    ]);
  });
});
