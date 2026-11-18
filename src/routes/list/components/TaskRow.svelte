<script lang="ts">
  import { GripVertical, X } from "@lucide/svelte";
  import StatusIndicator from "$lib/components/StatusIndicator.svelte";
  import TaskJira from "./TaskJira.svelte";
  import { dragHandle } from "svelte-dnd-action";
  import type { Task } from "../state/types";
  import { getBoard } from "../state/context";

  const board = getBoard();
  let { task }: { task: Task } = $props();
</script>

<div class={["group/task relative flex min-h-16 items-start gap-2 rounded-lg border p-3 hover:border-primary", board.selectedTask?.id === task.id && "ring-2 ring-primary", task.status === "done" ? "border-completed-border bg-completed-bg" : "border-border bg-surface"]}>
  <span
    use:dragHandle
    role="button"
    tabindex="0"
    class="relative z-10 inline-flex min-h-8 shrink-0 cursor-grab touch-none items-center justify-center rounded text-muted select-none hover:text-primary active:cursor-grabbing focus-visible:text-primary"
    aria-label={`Move ${task.title}`}
    title="Drag to move · Space to pick up, arrows to reorder"
    ><GripVertical size={15} /></span
  >
  <div class="min-w-0 flex-1">
    <button
      class="block w-full rounded text-left after:absolute after:inset-0 after:rounded-lg"
      data-task-details-id={task.id}
      aria-label={`Open details for ${task.title}`}
      aria-expanded={board.selectedTask?.id === task.id}
      aria-controls={board.selectedTask?.id === task.id ? "task-details" : undefined}
      onclick={() => board.openTaskDetails(task.id)}
    >
      <span class="flex items-start gap-2">
        {#if task.status === "done"}
          <span class="mt-2 inline-flex shrink-0" title="Completed"><StatusIndicator status={task.status} /><span class="sr-only">Completed:</span></span>
        {/if}
        <span class={["text-sm leading-relaxed wrap-anywhere", task.status === "done" && "text-muted line-through"]}>{task.title}</span>
      </span>
      {#if task.description}
        <span class="mt-1 block line-clamp-2 text-xs leading-relaxed whitespace-pre-wrap text-muted wrap-anywhere">{task.description}</span>
      {/if}
    </button>
    <div class="relative z-10"><TaskJira {task} /></div>
  </div>
  <button
    class="relative z-10 inline-flex size-8 shrink-0 items-center justify-center rounded-md text-muted opacity-0 group-hover/task:opacity-100 group-focus-within/task:opacity-100 hover:bg-surface-hover hover:text-error focus-visible:opacity-100 pointer-coarse:opacity-100 max-[580px]:opacity-100"
    aria-label={`Remove ${task.title}`}
    onclick={() => { void board.deleteTask(task.id); }}><X size={15} /></button
  >
</div>
