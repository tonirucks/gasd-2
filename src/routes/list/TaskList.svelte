<script lang="ts">
  import { flip } from "svelte/animate";
  import { dragHandleZone, type DndEvent } from "svelte-dnd-action";
  import TaskRow from "./TaskRow.svelte";
  import type { Task } from "./types";

  let {
    tasks,
    label,
    flipDurationMs,
    onreorder,
    onplace,
    ontoggle,
    onremove,
    onedit,
  }: {
    tasks: Task[];
    label: string;
    flipDurationMs: number;
    onreorder: (tasks: Task[]) => void;
    onplace: (tasks: Task[], id: number) => void;
    ontoggle: (taskId: number) => void;
    onremove: (taskId: number) => void;
    onedit: (taskId: number, title: string) => Promise<boolean>;
  } = $props();

  function reorder(event: CustomEvent<DndEvent<Task>>): void {
    onreorder(event.detail.items);
  }
</script>

<div
  class="rounded-[5px] py-1"
  use:dragHandleZone={{
    items: tasks,
    type: "tasks",
    useCursorForDetection: true,
    flipDurationMs,
    dropTargetStyle: {},
    delayTouchStart: true,
  }}
  onconsider={reorder}
  onfinalize={(event) => {
    reorder(event);
    const id = Number(event.detail.info.id);
    // Both source and destination finalize; only the destination saves the move.
    if (event.detail.items.some((task) => task.id === id)) {
      onplace(event.detail.items, id);
    }
  }}
  aria-label={label}
>
  {#each tasks as task (task.id)}
    <!-- Include the gutter handle in each draggable item's hit area. -->
    <div class="-ml-7 pl-7" aria-label={task.title} animate:flip={{ duration: flipDurationMs }}>
      <TaskRow
        {task}
        {ontoggle}
        {onremove}
        {onedit}
      />
    </div>
  {/each}
</div>
