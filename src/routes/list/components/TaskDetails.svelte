<script lang="ts">
  import { onMount, tick } from "svelte";
  import { MediaQuery } from "svelte/reactivity";
  import { Check, X } from "@lucide/svelte";
  import { getBoard } from "../state/context";
  import StatusIndicator from "$lib/components/StatusIndicator.svelte";

  const board = getBoard();
  const narrowScreen = new MediaQuery("(max-width: 900px)");
  const selectedTaskId = $derived(board.selectedTask?.id);
  let panel: HTMLElement;
  const saveStatus = $derived.by(() => {
    if (board.state.busy) return "Saving…";
    if (board.taskDirty) return "Unsaved changes";
    return "All changes saved";
  });

  $effect(() => {
    if (selectedTaskId !== undefined) panel?.focus();
  });

  onMount(() => {
    const opener = document.querySelector(`[data-task-details-id="${board.selectedTask?.id}"]`);
    return () => {
      void tick().then(() => {
        if (opener instanceof HTMLElement && opener.isConnected) opener.focus();
      });
    };
  });

  function handleKeydown(event: KeyboardEvent): void {
    if (event.key === "Escape" && !event.defaultPrevented) {
      event.preventDefault();
      board.closeTaskDetails();
    }
    if (event.key === "Tab" && narrowScreen.current) {
      const controls = panel.querySelectorAll<HTMLElement>("button:enabled, input:enabled, textarea:enabled");
      const first = controls[0];
      const last = controls[controls.length - 1];
      const active = document.activeElement;
      if (!first) {
        event.preventDefault();
      } else if (event.shiftKey && (active === first || active === panel)) {
        event.preventDefault();
        last?.focus();
      } else if (!event.shiftKey && (active === last || !panel.contains(active))) {
        event.preventDefault();
        first?.focus();
      }
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<aside
  id="task-details"
  bind:this={panel}
  tabindex="-1"
  role={narrowScreen.current ? "dialog" : "complementary"}
  aria-modal={narrowScreen.current ? true : undefined}
  aria-labelledby="task-details-heading"
  class="fixed inset-y-0 right-0 z-40 flex w-104 max-w-full flex-col border-l border-border bg-surface shadow-xl outline-none max-[900px]:w-full"
>
  <header class="flex items-center justify-between border-b border-border px-6 py-5">
    <h2 id="task-details-heading" class="font-brand text-lg font-semibold">Task details</h2>
    <button
      class="inline-flex size-9 items-center justify-center rounded-md text-muted hover:bg-surface-hover hover:text-text"
      aria-label="Close task details"
      disabled={board.state.busy}
      onclick={board.closeTaskDetails}
    ><X size={18} /></button>
  </header>
  {#if board.taskDraft && board.selectedTask}
    <form class="flex min-h-0 flex-1 flex-col" onsubmit={(event) => { event.preventDefault(); void board.saveTaskDetails(); }}>
      <fieldset disabled={board.state.busy} class="min-h-0 flex-1 space-y-6 overflow-y-auto p-6">
        <div class="flex items-center gap-2 text-xs text-muted">
          <StatusIndicator status={board.selectedTask.status} />
          <span>{board.sections.find((section) => section.id === board.selectedTask?.sectionId)?.name}</span>
          <span aria-hidden="true">·</span>
          <span>{{ open: "Open", inProgress: "In progress", waiting: "Waiting", done: "Done" }[board.selectedTask.status]}</span>
        </div>
        <label class="block text-sm font-medium">
          Title
          <input
            class="mt-2 w-full rounded-md border border-border bg-base px-3 py-2 text-sm focus:border-primary"
            bind:value={board.taskDraft.title}
            required
          />
        </label>
        <label class="block text-sm font-medium">
          Description <span class="font-normal text-muted">(optional)</span>
          <textarea
            class="mt-2 block min-h-56 w-full resize-y rounded-md border border-border bg-base px-3 py-2 text-sm leading-relaxed placeholder:text-muted focus:border-primary"
            bind:value={board.taskDraft.description}
            rows="10"
            placeholder="Add more details…"
          ></textarea>
        </label>
        {#if board.state.error}
          <p role="alert" class="text-sm text-error">{board.state.error}</p>
        {/if}
      </fieldset>
      <footer class="flex items-center justify-between gap-3 border-t border-border px-6 py-4">
        <p class="text-xs text-muted" role="status">{saveStatus}</p>
        <div class="flex shrink-0 items-center gap-2">
          <button class="rounded-md px-3 py-2 text-sm text-muted hover:bg-surface-hover" type="button" disabled={board.state.busy} onclick={board.closeTaskDetails}>Cancel</button>
          <button class="button-primary text-sm" type="submit" disabled={board.state.busy || !board.taskDirty || !board.taskDraft.title.trim()}><Check size={16} /> Save</button>
        </div>
      </footer>
    </form>
  {/if}
</aside>
