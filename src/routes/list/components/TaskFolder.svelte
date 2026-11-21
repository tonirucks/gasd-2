<script lang="ts">
  import { onMount } from "svelte";
  import { Folder, Unlink } from "@lucide/svelte";
  import { folderClient } from "$lib/data/client";
  import { searchFolders } from "$lib/data/folder-search";
  import type { Task } from "$lib/data/types";
  import { getBoard } from "../state/context";

  let { task }: { task: Task } = $props();
  const board = getBoard();
  let folders = $state<string[]>([]);
  let query = $state("");
  let editing = $state(false);
  let loading = $state(false);
  let error = $state("");
  let editorError = $state("");
  let openingEditor = $state(false);
  const matches = $derived(searchFolders(folders, query));

  async function openEditor(editor: "vscode" | "zed") {
    editorError = "";
    openingEditor = true;
    try { await folderClient.openEditor(task.id, editor); }
    catch (error) {
      editorError = typeof error === "object" && error !== null && "message" in error
        ? String(error.message) : "Could not open the editor.";
    } finally { openingEditor = false; }
  }

  async function loadFolders() {
    loading = true;
    error = "";
    try { folders = await folderClient.get(); }
    catch { error = "Could not load linked folders. Please try again."; }
    finally { loading = false; }
  }

  onMount(() => { void loadFolders(); });

  async function link(folder: string) {
    if (await board.linkFolder(task.id, folder)) {
      editing = false;
      query = "";
      void loadFolders();
    }
  }
</script>

<section class="mt-6 border-t border-border pt-6" aria-labelledby="task-folder-heading">
  <h3 id="task-folder-heading" class="mb-3 text-sm font-medium">Folder</h3>
  {#if task.folder}
    <div class="mb-3 flex items-start gap-2 text-sm">
      <Folder size={16} class="mt-0.5 shrink-0 text-muted" />
      <span class="min-w-0 flex-1 break-all">{task.folder}</span>
      <button type="button" class="shrink-0 rounded p-1 text-muted hover:text-primary disabled:opacity-50" disabled={openingEditor || board.state.busy} aria-label="Open folder in VS Code Insiders" title="Open in VS Code Insiders" onclick={() => openEditor("vscode")}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="m17.583 2.093 5.734 2.76A1.25 1.25 0 0 1 24 5.98v12.04a1.25 1.25 0 0 1-.683 1.127l-5.734 2.76a1.25 1.25 0 0 1-1.419-.243L6.74 13.08l-4.105 3.115a.833.833 0 0 1-1.064-.047L.275 14.97a.833.833 0 0 1 0-1.23L3.835 12 .275 10.26a.833.833 0 0 1 0-1.23l1.296-1.178a.833.833 0 0 1 1.064-.047L6.74 10.92l9.424-8.584a1.25 1.25 0 0 1 1.419-.243ZM18 6.5 10.5 12 18 17.5Z" /></svg>
      </button>
      <button type="button" class="shrink-0 rounded p-1 text-muted hover:text-primary disabled:opacity-50" disabled={openingEditor || board.state.busy} aria-label="Open folder in Zed" title="Open in Zed" onclick={() => openEditor("zed")}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M2.25 1.5a.75.75 0 0 0-.75.75v16.5H0V2.25A2.25 2.25 0 0 1 2.25 0h20.095c1.002 0 1.504 1.212.795 1.92L10.764 14.298h3.486V12.75h1.5v1.922a1.125 1.125 0 0 1-1.125 1.125H9.264l-2.578 2.578h11.689V9h1.5v9.375a1.5 1.5 0 0 1-1.5 1.5H5.185L2.562 22.5H21.75a.75.75 0 0 0 .75-.75V5.25H24v16.5A2.25 2.25 0 0 1 21.75 24H1.655C.653 24 .151 22.788.86 22.08L13.19 9.75H9.75v1.5h-1.5V9.375A1.125 1.125 0 0 1 9.375 8.25h5.314l2.625-2.625H5.625V15h-1.5V5.625a1.5 1.5 0 0 1 1.5-1.5h13.19L21.438 1.5z" /></svg>
      </button>
      <button type="button" class="rounded p-1 text-muted hover:text-error" disabled={board.state.busy} aria-label="Unlink folder" onclick={() => board.linkFolder(task.id, null)}><Unlink size={14} /></button>
    </div>
    {#if editorError}<p role="alert" class="mb-3 text-xs text-error">{editorError}</p>{/if}
  {/if}
  {#if editing}
    <label class="block text-xs text-muted">
      Search previously linked folders
      <input type="search" class="mt-2 w-full rounded-md border border-border bg-base px-3 py-2 text-sm text-text focus:border-primary" placeholder="Fuzzy search by name or path…" bind:value={query} />
    </label>
    <ul aria-label="Previously linked folders" class="mt-2 max-h-48 overflow-y-auto rounded-md border border-border">
      {#each matches as folder (folder)}
        <li><button type="button" class="w-full break-all px-3 py-2 text-left text-xs hover:bg-surface-hover focus:bg-surface-hover" disabled={board.state.busy} aria-pressed={task.folder === folder} onclick={() => link(folder)}>{folder}</button></li>
      {/each}
    </ul>
    {#if loading}
      <p class="mt-2 text-xs text-muted" role="status">Loading folders…</p>
    {:else if !matches.length}
      <p class="mt-2 text-xs text-muted">{folders.length ? "No matching folders." : "No folders linked yet."}</p>
    {/if}
    <form class="mt-3 space-y-2" onsubmit={(event) => { event.preventDefault(); if (query) void link(query); }}>
      <button type="submit" class="button-primary text-xs" disabled={board.state.busy || !query}>Link entered path</button>
      <button type="button" class="ml-2 rounded-md px-2 py-1 text-xs text-muted hover:bg-surface-hover" disabled={board.state.busy} onclick={() => editing = false}>Cancel</button>
    </form>
  {:else}
    <button type="button" class="rounded-md border border-border px-3 py-2 text-sm hover:border-primary hover:text-primary" disabled={board.state.busy} onclick={() => { editing = true; void loadFolders(); }}>{task.folder ? "Change folder" : "Link folder"}</button>
  {/if}
  {#if error}
    <p role="alert" class="mt-2 text-xs text-error">{error}</p>
    <button type="button" class="mt-1 text-xs text-primary" disabled={loading} onclick={loadFolders}>Retry loading folders</button>
  {/if}
</section>
