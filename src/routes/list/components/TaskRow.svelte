<script lang="ts">
  import StatusIndicator from "$lib/components/StatusIndicator.svelte";
  import TaskJira from "./TaskJira.svelte";
  import TaskSubtasks from "./TaskSubtasks.svelte";
  import type { Task } from "../state/types";
  import { getBoard } from "../state/context";

  const board = getBoard();
  let { task }: { task: Task } = $props();

  // Keep secondary controls' pointer and keyboard interactions out of card dragging.
  function preventDrag(node: HTMLElement) {
    const stop = (event: Event) => event.stopPropagation();
    node.addEventListener("mousedown", stop);
    node.addEventListener("touchstart", stop);
    node.addEventListener("keydown", stop);
    return {
      destroy() {
        node.removeEventListener("mousedown", stop);
        node.removeEventListener("touchstart", stop);
        node.removeEventListener("keydown", stop);
      },
    };
  }
</script>

<div class={["relative min-h-16 cursor-grab rounded-lg border p-3 hover:border-primary active:cursor-grabbing", board.selectedTask?.id === task.id && "ring-2 ring-primary", task.status === "done" ? "border-completed-border bg-completed-bg" : "border-border bg-surface"]}>
  <div class="min-w-0 flex-1">
    <button
      class="block w-full cursor-grab rounded text-left active:cursor-grabbing"
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
        <span class={["text-sm font-medium leading-relaxed wrap-anywhere", task.status === "done" && "text-muted line-through"]}>{task.title}</span>
      </span>
      {#if task.description}
        <span class="mt-1 block line-clamp-2 text-xs leading-relaxed whitespace-pre-wrap text-muted wrap-anywhere">{task.description}</span>
      {/if}
    </button>
    {#if task.jira}
      <div use:preventDrag class="relative z-10 [&_button]:cursor-pointer"><TaskJira {task} compact /></div>
    {/if}
    {#if task.subtasks.length > 0}
      <div use:preventDrag class="cursor-auto">
        <TaskSubtasks {task} compact />
      </div>
    {/if}
  </div>
</div>
