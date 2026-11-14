<script lang="ts">
  import { GripVertical, Pencil, X } from "@lucide/svelte";
  import InlineForm from "./InlineForm.svelte";
  import { dragHandle } from "svelte-dnd-action";
  import type { Task } from "./types";
  import { getBoard } from "./context";

  const board = getBoard();
  let { task }: { task: Task } = $props();
  let editing = $state(false);
  let title = $state("");
</script>

<div class="group/task relative flex min-h-16 items-start gap-2 rounded-md border border-surface bg-[var(--base)] p-3 shadow-sm">
  <span
    use:dragHandle
    role="button"
    tabindex="0"
    class="mt-0.5 inline-flex shrink-0 cursor-grab touch-none items-center justify-center rounded text-muted select-none hover:text-mauve active:cursor-grabbing focus-visible:text-mauve"
    aria-label={`Move ${task.title}`}
    title="Drag to move · Space to pick up, arrows to reorder"
    ><GripVertical size={15} /></span
  >
  {#if editing}
    <InlineForm
      variant="rename"
      bind:value={title}
      label="Task title"
      saveLabel="Save task title"
      cancelLabel="Cancel task edit"
      onsave={async (value) => { if (await board.editTask(task.id, value)) editing = false; }}
      oncancel={() => (editing = false)}
    />
  {:else}
    <span class={["min-w-0 flex-1 text-[13px] leading-[1.6] wrap-anywhere", task.status === "done" && "text-muted line-through"]}>{task.title}</span>
    <button
      class="inline-flex shrink-0 rounded p-1.25 text-muted opacity-0 group-hover/task:opacity-100 hover:bg-surface focus-visible:opacity-100 max-[580px]:opacity-100"
      aria-label={`Edit ${task.title}`}
      onclick={() => { title = task.title; editing = true; }}><Pencil size={12} /></button>
  {/if}
  <button
    class="inline-flex shrink-0 items-center justify-center rounded p-1.25 text-muted opacity-0 group-hover/task:opacity-100 hover:bg-surface hover:text-text focus-visible:opacity-100 max-[580px]:opacity-100"
    aria-label={`Remove ${task.title}`}
    onclick={() => { void board.deleteTask(task.id); }}><X size={15} /></button
  >
</div>
