import { get, writable } from "svelte/store";
import { settingsClient, type SettingsClient } from "./client";
import type { AppSettings } from "./types";

type SettingsState = {
  data: AppSettings | null;
  busy: boolean;
  error: string | null;
  notice: string | null;
};

/** Persisted settings never contain the API token; it is only an input to save. */
export function createSettingsStore(client: SettingsClient = settingsClient) {
  const state = writable<SettingsState>({ data: null, busy: false, error: null, notice: null });

  async function run(operation: () => Promise<AppSettings | void>, notice: string | null = null): Promise<boolean> {
    if (get(state).busy) return false;
    state.update((value) => ({ ...value, busy: true, error: null, notice: null }));
    try {
      const data = await operation();
      state.update((value) => ({ ...value, data: data ?? value.data, notice }));
      return true;
    } catch (cause) {
      const message = typeof cause === "object" && cause !== null && "message" in cause
        ? String(cause.message) : "Could not access settings. Please retry.";
      state.update((value) => ({ ...value, error: message }));
      return false;
    } finally {
      state.update((value) => ({ ...value, busy: false }));
    }
  }

  return {
    subscribe: state.subscribe,
    load: () => run(() => client.get()),
    saveJira: (site: string, email: string, token: string | null) =>
      run(() => client.saveJira(site, email, token), "Jira settings saved."),
    testJira: () => run(() => client.testJira(), "Connected to Jira successfully."),
    clearJira: () => run(async () => {
      await client.clearJira();
      return client.get();
    }, "Jira disconnected. Saved queries and task links were kept."),
  };
}
