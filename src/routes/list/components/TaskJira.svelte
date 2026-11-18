<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { ExternalLink, Link, Pencil, Unlink } from "@lucide/svelte";
  import type { Task } from "$lib/data/types";
  import { getBoard } from "../state/context";

  let { task, compact = false }: { task: Task; compact?: boolean } = $props();
  const board = getBoard();
  let editing = $state(false);
  let key = $state("");
  let error = $state("");

  function startEditing(): void {
    key = task.jira?.key ?? "";
    error = "";
    editing = true;
  }

  async function openTicket(): Promise<void> {
    if (!task.jira) return;
    error = "";
    try { await openUrl(task.jira.url); }
    catch { error = "Could not open the Jira ticket."; }
  }

  async function saveLink(): Promise<void> {
    if (!key.trim() || board.state.busy) return;
    if (await board.linkJira(task.id, key.trim())) editing = false;
  }
</script>

<div class={compact ? "mt-1 text-xs text-muted" : "text-sm"}>
  {#if editing && !compact}
    <form class="space-y-3" onsubmit={(event) => { event.preventDefault(); void saveLink(); }}>
      <label class="block text-xs text-muted">
        Ticket key
        <input class="mt-2 w-full rounded-md border border-border bg-base px-3 py-2 text-sm text-text focus:border-primary" aria-label="Jira ticket key" placeholder="TEAM-123" bind:value={key} disabled={board.state.busy} required />
      </label>
      <div class="flex items-center gap-2">
        <button class="button-primary text-sm" type="submit" disabled={board.state.busy || !key.trim()}>Save link</button>
        <button class="rounded-md px-3 py-2 text-sm text-muted hover:bg-surface-hover" type="button" disabled={board.state.busy} onclick={() => editing = false}>Cancel</button>
      </div>
    </form>
  {:else}
    {#if task.jira}
      <div class="flex flex-wrap items-center gap-2">
        <button type="button" class="inline-flex items-center gap-1.5 rounded text-primary hover:underline" onclick={openTicket} title="Open Jira ticket">{task.jira.key}<ExternalLink size={compact ? 12 : 14} /></button>
        {#if !compact}
          <div class="ml-auto flex items-center gap-1">
            <button type="button" class="inline-flex items-center gap-1.5 rounded-md px-2 py-1.5 text-xs text-muted hover:bg-surface-hover hover:text-text" disabled={board.state.busy} onclick={startEditing}><Pencil size={13} /> Change</button>
            <button type="button" class="inline-flex items-center gap-1.5 rounded-md px-2 py-1.5 text-xs text-muted hover:bg-surface-hover hover:text-error" disabled={board.state.busy} aria-label={`Unlink ${task.jira.key}`} onclick={() => board.linkJira(task.id, null)}><Unlink size={13} /> Unlink</button>
          </div>
        {/if}
      </div>
    {:else if !compact}
      <p class="mb-3 text-xs text-muted">Connect this task to a Jira ticket.</p>
      <button type="button" class="inline-flex items-center gap-2 rounded-md border border-border px-3 py-2 text-sm hover:border-primary hover:text-primary" disabled={board.state.busy} onclick={startEditing}><Link size={14} /> Link Jira ticket</button>
    {/if}
  {/if}
  {#if error}<p role="alert" class="mt-2 text-xs text-error">{error}</p>{/if}
</div>
