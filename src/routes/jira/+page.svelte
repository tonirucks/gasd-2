<script lang="ts">
  import { onMount } from "svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { jiraClient } from "$lib/data/client";
  import { createListStore } from "$lib/data/list-store";
  import type { JiraQuery, JiraTicket } from "$lib/data/types";

  const list = createListStore();
  let queries = $state<JiraQuery[]>([]);
  let selected = $state<number | null>(null);
  let editing = $state(false);
  let editId = $state<number | null>(null);
  let name = $state("");
  let jql = $state("");
  let tickets = $state<JiraTicket[]>([]);
  let nextPageToken = $state<string | null>(null);
  let searched = $state(false);
  let busy = $state(false);
  let error = $state("");

  async function run(operation: () => Promise<void>): Promise<void> {
    if (busy) return;
    busy = true;
    error = "";
    try { await operation(); }
    catch (cause) {
      error = typeof cause === "object" && cause !== null && "message" in cause
        ? String(cause.message) : "Could not access Jira. Please retry.";
    } finally { busy = false; }
  }

  async function loadQueries(): Promise<void> {
    queries = await jiraClient.queries();
    if (!queries.some((query) => query.id === selected)) {
      selected = queries[0]?.id ?? null;
      clearResults();
    }
  }

  function clearResults(): void {
    tickets = [];
    nextPageToken = null;
    searched = false;
  }

  function edit(query?: JiraQuery): void {
    editId = query?.id ?? null;
    name = query?.name ?? "";
    jql = query?.jql ?? "";
    editing = true;
  }

  async function saveQuery(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    await run(async () => {
      const saved = await jiraClient.saveQuery(editId, name, jql);
      selected = saved.id;
      await loadQueries();
      clearResults();
      editing = false;
    });
  }

  async function deleteQuery(): Promise<void> {
    if (selected === null || !window.confirm("Delete this saved query? Task links will be kept.")) return;
    const id = selected;
    await run(async () => {
      await jiraClient.deleteQuery(id);
      await loadQueries();
      clearResults();
    });
  }

  async function search(more = false): Promise<void> {
    if (selected === null) return;
    const id = selected;
    await run(async () => {
      if (!more) clearResults();
      const page = await jiraClient.search(id, more ? nextPageToken : null);
      const seen = new Set(tickets.map((ticket) => ticket.key));
      tickets = [...tickets, ...page.tickets.filter((ticket) => !seen.has(ticket.key))];
      nextPageToken = page.nextPageToken;
      searched = true;
    });
  }

  onMount(() => {
    void run(loadQueries);
    void list.load();
  });
</script>

<svelte:window onfocus={() => { void list.load(); }} />

<svelte:head>
  <title>Jira import · little by little</title>
  <meta name="description" content="Run saved Jira searches and link tickets to your local tasks." />
</svelte:head>

<main class="mx-auto max-w-280 px-7 pt-12 pb-20 max-[580px]:pt-9.5">
  <h1 class="font-brand text-3xl font-bold">Jira import</h1>
  <p class="mt-2 text-sm text-muted">Find Jira tickets with saved JQL queries and link them to your tasks. Jira access is read-only.</p>
  <p class="mt-3 text-xs text-muted"><a href="/settings" class="text-mauve hover:underline">Configure your Jira connection in Settings.</a></p>
  <section class="mt-8 rounded-lg border border-surface p-6" aria-labelledby="queries-heading">
  <h2 id="queries-heading" class="font-brand text-lg font-semibold">Saved queries</h2>
  {#if error}
    <div role="alert" class="mt-3 text-sm text-mauve">{error} <button class="underline" onclick={() => run(loadQueries)}>Reload queries</button></div>
  {/if}
  {#if $list.error}
    <div role="alert" class="mt-3 text-sm text-mauve">{$list.error} <button class="underline" disabled={$list.busy} onclick={list.load}>Reload tasks</button></div>
  {/if}
  <fieldset disabled={busy || $list.busy} class="mt-4 min-w-0 disabled:opacity-60" aria-busy={busy || $list.busy}>
    <div class="flex flex-wrap items-center gap-3 text-sm">
      <select class="max-w-full rounded border border-surface bg-[var(--base)] p-2" aria-label="Saved JQL query" bind:value={selected} onchange={clearResults}>
        {#if queries.length === 0}<option value={null}>No saved queries</option>{/if}
        {#each queries as query (query.id)}<option value={query.id}>{query.name}</option>{/each}
      </select>
      <button class="text-mauve" disabled={selected === null} onclick={() => search()}>Run query</button>
      <button onclick={() => edit()}>New query</button>
      {#if selected !== null}
        <button onclick={() => edit(queries.find((query) => query.id === selected))}>Edit query</button>
        <button onclick={deleteQuery}>Delete query</button>
      {/if}
    </div>
    {#if editing}
      <form class="mt-4 flex flex-col gap-3" onsubmit={saveQuery}>
        <label class="text-xs">Query name
          <input class="mt-1 block w-full rounded border border-surface bg-transparent p-2 text-sm" bind:value={name} required />
        </label>
        <label class="text-xs">JQL
          <textarea class="mt-1 block w-full rounded border border-surface bg-transparent p-2 font-mono text-sm" rows="3" placeholder="assignee = currentUser() AND resolution = Unresolved ORDER BY updated DESC" bind:value={jql} required></textarea>
        </label>
        <div class="flex gap-3 text-sm"><button type="submit" class="text-mauve">Save query</button><button type="button" onclick={() => editing = false}>Cancel</button></div>
      </form>
    {/if}
    {#if searched && tickets.length === 0}<p class="mt-4 text-sm text-muted">No tickets match this query.</p>{/if}
    <ul class="mt-4 flex flex-col gap-3">
      {#each tickets as ticket (ticket.key)}
        <li class="flex flex-wrap items-center gap-3 border-t border-surface pt-3 text-sm">
          <button class="text-mauve hover:underline" onclick={() => run(async () => { await openUrl(ticket.url); })}>{ticket.key} ↗</button>
          <span class="min-w-0 flex-1 wrap-anywhere">{ticket.summary}</span>
          <span class="text-xs text-muted">{ticket.status}</span>
          <select class="max-w-56 rounded border border-surface bg-[var(--base)] p-2 text-xs" aria-label={`Link ${ticket.key} to a task`} value="" onchange={(event) => {
            const id = Number(event.currentTarget.value);
            event.currentTarget.value = "";
            if (!id) return;
            const task = $list.data.tasks.find((task) => task.id === id);
            if (task?.jira && !window.confirm(`Replace ${task.jira.key} with ${ticket.key} on “${task.title}”?`)) return;
            void list.linkJira(id, ticket.key);
          }}>
            <option value="">Link to task…</option>
            {#each $list.data.tasks as task (task.id)}
              <option value={task.id}>{task.title}{task.jira ? ` (${task.jira.key})` : ""}</option>
            {/each}
          </select>
        </li>
      {/each}
    </ul>
    {#if nextPageToken}<button class="mt-4 text-sm text-mauve" onclick={() => search(true)}>Load more tickets</button>{/if}
  </fieldset>
  <p role="status" class="mt-2 text-xs text-muted">{busy ? "Loading Jira…" : ""}</p>
  <p role="status" class="text-xs text-muted">{$list.busy ? "Loading / updating tasks…" : ""}</p>
  {#if $list.loaded && $list.data.tasks.length === 0}
    <p class="mt-3 text-sm text-muted">No local tasks to link yet. <a href="/" class="text-mauve hover:underline">Create a task on the board.</a></p>
  {/if}
  </section>
</main>
