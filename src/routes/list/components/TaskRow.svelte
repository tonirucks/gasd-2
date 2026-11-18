<script lang="ts">
  import { GripVertical, Pencil, X } from "@lucide/svelte";
  import StatusIndicator from "$lib/components/StatusIndicator.svelte";
  import InlineForm from "./InlineForm.svelte";
  import TaskJira from "./TaskJira.svelte";
  import { dragHandle } from "svelte-dnd-action";
  import type { Task } from "../state/types";
  import { getBoard } from "../state/context";

  const board = getBoard();
  let { task }: { task: Task } = $props();
  let editing = $state(false);
  let title = $state("");
</script>

<div class={["group/task relative flex min-h-16 items-start gap-2 rounded-lg border p-3", task.status === "done" ? "border-completed-border bg-completed-bg" : "border-border bg-surface"]}>
  <span
    use:dragHandle
    role="button"
    tabindex="0"
    class="inline-flex min-h-8 shrink-0 cursor-grab touch-none items-center justify-center rounded text-muted select-none hover:text-primary active:cursor-grabbing focus-visible:text-primary"
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
    <div class="min-w-0 flex-1">
      <span class="flex items-start gap-2">
        {#if task.status === "done"}
          <span class="mt-2 inline-flex shrink-0" title="Completed"><StatusIndicator status={task.status} /><span class="sr-only">Completed:</span></span>
        {/if}
        <span class={["text-sm leading-relaxed wrap-anywhere", task.status === "done" && "text-muted line-through"]}>{task.title}</span>
      </span>
      <TaskJira {task} />
    </div>
    <button
      class="inline-flex size-8 shrink-0 items-center justify-center rounded-md text-muted opacity-0 group-hover/task:opacity-100 group-focus-within/task:opacity-100 hover:bg-primary-subtle hover:text-primary focus-visible:opacity-100 pointer-coarse:opacity-100 max-[580px]:opacity-100"
      aria-label={`Edit ${task.title}`}
      onclick={() => { title = task.title; editing = true; }}><Pencil size={12} /></button>
  {/if}
  <button
    class="inline-flex size-8 shrink-0 items-center justify-center rounded-md text-muted opacity-0 group-hover/task:opacity-100 group-focus-within/task:opacity-100 hover:bg-surface-hover hover:text-error focus-visible:opacity-100 pointer-coarse:opacity-100 max-[580px]:opacity-100"
    aria-label={`Remove ${task.title}`}
    onclick={() => { void board.deleteTask(task.id); }}><X size={15} /></button
  >
</div>
