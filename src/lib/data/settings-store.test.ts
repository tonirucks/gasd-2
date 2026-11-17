import { get } from "svelte/store";
import { describe, expect, it, vi } from "vitest";
import { createSettingsStore } from "./settings-store";

function setup() {
  const data = { jira: { site: "https://team.atlassian.net", email: "me@example.com", hasToken: true } };
  const client = {
    get: vi.fn().mockResolvedValue(data),
    saveJira: vi.fn().mockResolvedValue(data),
    clearJira: vi.fn().mockResolvedValue(undefined),
    testJira: vi.fn().mockResolvedValue(undefined),
  };
  return { data, client, store: createSettingsStore(client) };
}

describe("settings store", () => {
  it("loads settings and saves tokens without retaining them in state", async () => {
    const { data, client, store } = setup();
    expect(await store.load()).toBe(true);
    expect(get(store).data).toEqual(data);
    expect(await store.saveJira(data.jira.site, data.jira.email, "secret-token")).toBe(true);
    expect(client.saveJira).toHaveBeenCalledWith(data.jira.site, data.jira.email, "secret-token");
    expect(JSON.stringify(get(store))).not.toContain("secret-token");
    expect(get(store).notice).toBe("Jira settings saved.");
    await store.saveJira(data.jira.site, data.jira.email, null);
    expect(client.saveJira).toHaveBeenLastCalledWith(data.jira.site, data.jira.email, null);
  });

  it("keeps known settings on failure and supports recovery", async () => {
    const { data, client, store } = setup();
    client.get.mockRejectedValueOnce({ message: "Keychain locked" });
    expect(await store.load()).toBe(false);
    expect(get(store)).toMatchObject({ data: null, error: "Keychain locked", busy: false });
    await store.load();
    client.saveJira.mockRejectedValueOnce({ message: "Could not save" });
    expect(await store.saveJira("https://new.atlassian.net", "new@example.com", "secret")).toBe(false);
    expect(get(store)).toMatchObject({ data, error: "Could not save", notice: null, busy: false });
    expect(await store.testJira()).toBe(true);
    expect(get(store)).toMatchObject({ error: null, notice: "Connected to Jira successfully." });
  });

  it("clears the connection and reloads persisted settings", async () => {
    const { client, store } = setup();
    await store.load();
    const empty = { jira: { site: "", email: "", hasToken: false } };
    client.get.mockResolvedValueOnce(empty);
    expect(await store.clearJira()).toBe(true);
    expect(client.clearJira).toHaveBeenCalledOnce();
    expect(get(store).data).toEqual(empty);
  });

  it("prevents overlapping settings operations", async () => {
    const { client, store } = setup();
    let finish!: () => void;
    client.testJira.mockImplementationOnce(() => new Promise<void>((resolve) => { finish = resolve; }));
    const pending = store.testJira();
    expect(await store.clearJira()).toBe(false);
    expect(client.clearJira).not.toHaveBeenCalled();
    finish();
    expect(await pending).toBe(true);
  });
});
