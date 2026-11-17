<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import type { Task } from "$lib/data/types";
  import { getBoard } from "../state/context";

  let { task }: { task: Task } = $props();
  const board = getBoard();
  let editing = $state(false);
  let key = $state("");
  let error = $state("");

  async function openTicket(): Promise<void> {
    if (!task.jira) return;
    try { await openUrl(task.jira.url); }
    catch { error = "Could not open the Jira ticket."; }
  }
</script>

<div class="mt-1 text-xs text-muted">
  {#if editing}
    <form class="flex flex-wrap items-center gap-2" onsubmit={async (event) => {
      event.preventDefault();
      if (await board.linkJira(task.id, key.trim())) editing = false;
    }}>
      <input class="w-28 rounded border border-surface bg-transparent px-2 py-1 text-text" aria-label="Jira ticket key" placeholder="TEAM-123" bind:value={key} required />
      <button class="text-mauve" type="submit">Link</button>
      <button type="button" onclick={() => editing = false}>Cancel</button>
    </form>
  {:else}
    {#if task.jira}
      <button class="text-mauve hover:underline" onclick={openTicket} title="Open Jira ticket">{task.jira.key} ↗</button>
      <button class="ml-2 hover:text-text" aria-label={`Unlink ${task.jira.key}`} onclick={() => board.linkJira(task.id, null)}>Unlink</button>
    {:else}
      <button class="hover:text-mauve" onclick={() => { key = ""; editing = true; }}>Link Jira ticket</button>
    {/if}
  {/if}
  {#if error}<p role="alert">{error}</p>{/if}
</div>
